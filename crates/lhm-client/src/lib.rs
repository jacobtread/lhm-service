use interprocess::os::windows::named_pipe::{pipe_mode, tokio::DuplexPipeStream};
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use lhm_shared::*;

/// Client for accessing the LHM pipe
pub struct LHMClient {
    pipe: DuplexPipeStream<pipe_mode::Bytes>,
}

impl LHMClient {
    /// Connect to the pipe
    pub async fn connect() -> std::io::Result<Self> {
        let pipe = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(PIPE_NAME).await?;
        Ok(Self { pipe })
    }

    async fn send(&mut self, request: PipeRequest) -> std::io::Result<()> {
        let data_bytes = rmp_serde::to_vec(&request)
            .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
        let length = data_bytes.len() as u32;
        let length_bytes = length.to_be_bytes();

        // Write the length
        self.pipe.write_all(&length_bytes).await?;
        // Write the actual message
        self.pipe.write_all(&data_bytes).await?;

        // Flush the whole message
        self.pipe.flush().await?;

        Ok(())
    }

    async fn recv(&mut self) -> std::io::Result<PipeResponse> {
        let mut len_buffer = [0u8; 4];

        // Read the length of the payload
        self.pipe.read_exact(&mut len_buffer).await?;
        let length = u32::from_be_bytes(len_buffer) as usize;

        // Read the entire payload
        let mut data_buffer = vec![0u8; length];
        self.pipe.read_exact(&mut data_buffer).await?;

        let response: PipeResponse = rmp_serde::from_slice(&data_buffer)
            .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
        Ok(response)
    }

    /// Set the options for the computer (Which information to request)
    pub async fn set_options(&mut self, options: ComputerOptions) -> std::io::Result<()> {
        self.send(PipeRequest::SetOptions { options }).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Request and update all hardware and sensors
    ///
    /// This is required before you can call any querying or getter
    /// functions for hardware or sensors
    pub async fn update_all(&mut self) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateAll).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Requests a specific hardware item using its identifier
    ///
    /// You must call [Self::update_all] at least once before
    /// you will get a [Some] value
    pub async fn get_hardware_by_id(&mut self, id: String) -> std::io::Result<Option<Hardware>> {
        self.send(PipeRequest::GetHardwareById { id }).await?;
        match self.recv().await? {
            PipeResponse::Hardware { hardware } => Ok(hardware),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Queries the currently loaded selection of hardware
    ///
    /// `parent_id` Filters only to hardware are children of a hardware with a specific ID
    /// `ty` Filters only to hardware of a specific type
    pub async fn query_hardware(
        &mut self,
        parent_id: Option<String>,
        ty: Option<HardwareType>,
    ) -> std::io::Result<Vec<Hardware>> {
        self.send(PipeRequest::QueryHardware { parent_id, ty })
            .await?;
        match self.recv().await? {
            PipeResponse::Hardwares { hardware } => Ok(hardware),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Updates a specific hardware item by ID
    pub async fn update_hardware_by_id(&mut self, id: String) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateHardwareById { id }).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Updates a specific hardware item using its cache index.
    ///
    /// This is more efficient than sending the large identifier string
    /// in case where you are repeatedly calling update
    ///
    /// Note: Cache indexes will change each time [Self::update_all] is called
    /// you must ensure you obtain the latest cache index.
    pub async fn update_hardware_by_idx(&mut self, idx: usize) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateHardwareByIndex { idx })
            .await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Get a specific sensor by ID
    pub async fn get_sensor_by_id(&mut self, id: String) -> std::io::Result<Option<Sensor>> {
        self.send(PipeRequest::GetSensorById { id }).await?;
        match self.recv().await? {
            PipeResponse::Sensor { sensor } => Ok(sensor),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Get the value of a specific sensor by ID
    ///
    /// If `update` true is provided the sensor will be updated before
    /// querying   
    pub async fn get_sensor_value_by_id(
        &mut self,
        id: String,
        update: bool,
    ) -> std::io::Result<Option<f32>> {
        self.send(PipeRequest::GetSensorValueById { id, update })
            .await?;
        match self.recv().await? {
            PipeResponse::SensorValue { value } => Ok(value),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Get the value a specific sensor item using its cache index.
    ///
    /// This is more efficient than sending the large identifier string
    /// in case where you are repeatedly loading the value
    ///
    /// Note: Cache indexes will change each time [Self::update_all] is called
    /// you must ensure you obtain the latest cache index.
    pub async fn get_sensor_value_by_idx(
        &mut self,
        idx: usize,
        update: bool,
    ) -> std::io::Result<Option<f32>> {
        self.send(PipeRequest::GetSensorValueByIndex { idx, update })
            .await?;
        match self.recv().await? {
            PipeResponse::SensorValue { value } => Ok(value),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Queries the currently loaded selection of sensors
    ///
    /// `parent_id` Filters only to sensors are children of a hardware with a specific ID
    /// `ty` Filters only to sensor of a specific type
    pub async fn query_sensors(
        &mut self,
        parent_id: Option<String>,
        ty: Option<SensorType>,
    ) -> std::io::Result<Vec<Sensor>> {
        self.send(PipeRequest::QuerySensors { parent_id, ty })
            .await?;
        match self.recv().await? {
            PipeResponse::Sensors { sensors } => Ok(sensors),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Updates a specific sensor item by ID
    pub async fn update_sensor_by_id(&mut self, id: String) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateSensorById { id }).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    /// Updates a specific sensor item using its cache index.
    ///
    /// This is more efficient than sending the large identifier string
    /// in case where you are repeatedly calling update
    ///
    /// Note: Cache indexes will change each time [Self::update_all] is called
    /// you must ensure you obtain the latest cache index.
    pub async fn update_sensor_by_idx(&mut self, idx: usize) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateSensorByIndex { idx }).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }
}
