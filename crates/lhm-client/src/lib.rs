use interprocess::os::windows::named_pipe::{pipe_mode, tokio::DuplexPipeStream};
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use lhm_shared::*;

pub struct LHMClient {
    pipe: DuplexPipeStream<pipe_mode::Bytes>,
}

impl LHMClient {
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

    pub async fn set_options(&mut self, options: ComputerOptions) -> std::io::Result<()> {
        self.send(PipeRequest::SetOptions { options }).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    pub async fn update_all(&mut self) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateAll).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    pub async fn get_hardware_by_id(&mut self, id: String) -> std::io::Result<Option<Hardware>> {
        self.send(PipeRequest::GetHardwareById { id }).await?;
        match self.recv().await? {
            PipeResponse::Hardware { hardware } => Ok(hardware),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

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

    pub async fn update_hardware_by_id(&mut self, id: String) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateHardwareById { id }).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    pub async fn get_sensor_by_id(&mut self, id: String) -> std::io::Result<Option<Sensor>> {
        self.send(PipeRequest::GetSensorById { id }).await?;
        match self.recv().await? {
            PipeResponse::Sensor { sensor } => Ok(sensor),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

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

    pub async fn update_sensor_by_id(&mut self, id: String) -> std::io::Result<()> {
        self.send(PipeRequest::UpdateSensorById { id }).await?;
        match self.recv().await? {
            PipeResponse::Success => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::{ComputerOptions, HardwareType, LHMClient, SensorType};

    #[tokio::test]
    #[ignore = "Requires the service to be running"]
    async fn main() {
        let mut client = LHMClient::connect().await.unwrap();

        client
            .set_options(ComputerOptions {
                controller_enabled: true,
                cpu_enabled: true,
                gpu_enabled: true,
                motherboard_enabled: true,
                ..Default::default()
            })
            .await
            .unwrap();

        client.update_all().await.unwrap();

        // Request all CPU hardware
        let cpus = client
            .query_hardware(None, Some(HardwareType::Cpu))
            .await
            .unwrap();

        let cpu = cpus.first().unwrap();

        // Request all CPU temperature sensors
        let cpu_temps = client
            .query_sensors(Some(cpu.identifier.clone()), Some(SensorType::Temperature))
            .await
            .unwrap();

        // Find the package temperature
        let temp_sensor = cpu_temps
            .iter()
            .find(|sensor| sensor.name.eq("CPU Package"))
            .expect("Missing cpu temp sensor");

        println!("CPU is initially {}°C", temp_sensor.value);

        for _ in 0..15 {
            // Get the current sensor value
            let value = client
                .get_sensor_value_by_id(temp_sensor.identifier.clone(), true)
                .await
                .unwrap()
                .expect("cpu temp sensor is now unavailable");

            println!("CPU is now {}°C", value);
            sleep(Duration::from_secs(1)).await;
        }
    }
}
