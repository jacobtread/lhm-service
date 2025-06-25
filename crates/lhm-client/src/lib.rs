use codec::LHMFrame;
use parking_lot::Mutex;
use pipe::{PipeFuture, PipeTx};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};
use thiserror::Error;
use tokio::{spawn, sync::oneshot};
use tokio_util::bytes::Bytes;

pub use lhm_shared::*;

mod pipe;

#[cfg(feature = "service")]
pub mod service;

/// Handle to send requests through a [LHMClient]
#[derive(Clone)]
pub struct LHMClientHandle {
    tx: PipeTx,
    subscriptions: Arc<Subscriptions>,
}

#[derive(Debug, Error)]
pub enum LHMClientError {
    #[error(transparent)]
    Encode(rmp_serde::encode::Error),
    #[error(transparent)]
    Decode(rmp_serde::decode::Error),
    #[error("client request handler has closed")]
    SendError,
    #[error("client response channel has closed")]
    RecvError,
    #[error("server error: {0}")]
    Server(String),
    #[error("unexpected message")]
    UnexpectedMessage,
}

impl LHMClientHandle {
    async fn send_request(&self, request: PipeRequest) -> Result<PipeResponse, LHMClientError> {
        let body = rmp_serde::to_vec(&request).map_err(LHMClientError::Encode)?;
        let body = Bytes::from(body);

        let (tx, rx) = oneshot::channel();
        let id = self.subscriptions.insert(tx);

        if self.tx.send(LHMFrame { id, body }).is_err() {
            self.subscriptions.remove(id);
            return Err(LHMClientError::SendError);
        }

        let frame = rx.await.map_err(|_| LHMClientError::RecvError)?;
        let msg: PipeResponse =
            rmp_serde::from_slice(&frame.body).map_err(LHMClientError::Decode)?;

        match msg {
            PipeResponse::Error { error } => Err(LHMClientError::Server(error)),
            msg => Ok(msg),
        }
    }

    /// Set the options for the computer (Which information to request)
    pub async fn set_options(&self, options: ComputerOptions) -> Result<(), LHMClientError> {
        match self
            .send_request(PipeRequest::SetOptions { options })
            .await?
        {
            PipeResponse::Success => Ok(()),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Request and update all hardware and sensors
    ///
    /// This is required before you can call any querying or getter
    /// functions for hardware or sensors
    pub async fn update_all(&self) -> Result<(), LHMClientError> {
        match self.send_request(PipeRequest::UpdateAll).await? {
            PipeResponse::Success => Ok(()),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Requests a specific hardware item using its identifier
    ///
    /// You must call [Self::update_all] at least once before
    /// you will get a [Some] value
    pub async fn get_hardware_by_id(&self, id: String) -> Result<Option<Hardware>, LHMClientError> {
        match self
            .send_request(PipeRequest::GetHardwareById { id })
            .await?
        {
            PipeResponse::Hardware { hardware } => Ok(hardware),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Queries the currently loaded selection of hardware
    ///
    /// `parent_id` Filters only to hardware are children of a hardware with a specific ID
    /// `ty` Filters only to hardware of a specific type
    pub async fn query_hardware(
        &self,
        parent_id: Option<Option<String>>,
        ty: Option<HardwareType>,
    ) -> Result<Vec<Hardware>, LHMClientError> {
        match self
            .send_request(PipeRequest::QueryHardware { parent_id, ty })
            .await?
        {
            PipeResponse::Hardwares { hardware } => Ok(hardware),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Updates a specific hardware item by ID
    pub async fn update_hardware_by_id(&self, id: String) -> Result<(), LHMClientError> {
        match self
            .send_request(PipeRequest::UpdateHardwareById { id })
            .await?
        {
            PipeResponse::Success => Ok(()),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Updates a specific hardware item using its cache index.
    ///
    /// This is more efficient than sending the large identifier string
    /// in case where you are repeatedly calling update
    ///
    /// Note: Cache indexes will change each time [Self::update_all] is called
    /// you must ensure you obtain the latest cache index.
    pub async fn update_hardware_by_idx(&self, idx: usize) -> Result<(), LHMClientError> {
        match self
            .send_request(PipeRequest::UpdateHardwareByIndex { idx })
            .await?
        {
            PipeResponse::Success => Ok(()),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Get a specific sensor by ID
    pub async fn get_sensor_by_id(&self, id: String) -> Result<Option<Sensor>, LHMClientError> {
        match self.send_request(PipeRequest::GetSensorById { id }).await? {
            PipeResponse::Sensor { sensor } => Ok(sensor),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Get the value of a specific sensor by ID
    ///
    /// If `update` true is provided the sensor will be updated before
    /// querying   
    pub async fn get_sensor_value_by_id(
        &self,
        id: String,
        update: bool,
    ) -> Result<Option<f32>, LHMClientError> {
        match self
            .send_request(PipeRequest::GetSensorValueById { id, update })
            .await?
        {
            PipeResponse::SensorValue { value } => Ok(value),
            _ => Err(LHMClientError::UnexpectedMessage),
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
        &self,
        idx: usize,
        update: bool,
    ) -> Result<Option<f32>, LHMClientError> {
        match self
            .send_request(PipeRequest::GetSensorValueByIndex { idx, update })
            .await?
        {
            PipeResponse::SensorValue { value } => Ok(value),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Queries the currently loaded selection of sensors
    ///
    /// `parent_id` Filters only to sensors are children of a hardware with a specific ID
    /// `ty` Filters only to sensor of a specific type
    pub async fn query_sensors(
        &self,
        parent_id: Option<String>,
        ty: Option<SensorType>,
    ) -> Result<Vec<Sensor>, LHMClientError> {
        match self
            .send_request(PipeRequest::QuerySensors { parent_id, ty })
            .await?
        {
            PipeResponse::Sensors { sensors } => Ok(sensors),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Updates a specific sensor item by ID
    pub async fn update_sensor_by_id(&self, id: String) -> Result<(), LHMClientError> {
        match self
            .send_request(PipeRequest::UpdateSensorById { id })
            .await?
        {
            PipeResponse::Success => Ok(()),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }

    /// Updates a specific sensor item using its cache index.
    ///
    /// This is more efficient than sending the large identifier string
    /// in case where you are repeatedly calling update
    ///
    /// Note: Cache indexes will change each time [Self::update_all] is called
    /// you must ensure you obtain the latest cache index.
    pub async fn update_sensor_by_idx(&self, idx: usize) -> Result<(), LHMClientError> {
        match self
            .send_request(PipeRequest::UpdateSensorByIndex { idx })
            .await?
        {
            PipeResponse::Success => Ok(()),
            _ => Err(LHMClientError::UnexpectedMessage),
        }
    }
}

#[derive(Default)]
struct Subscriptions {
    id: AtomicU32,
    value: Mutex<HashMap<u32, oneshot::Sender<LHMFrame>>>,
}

impl Subscriptions {
    pub fn insert(&self, tx: oneshot::Sender<LHMFrame>) -> u32 {
        let id = self.id.fetch_add(1, Ordering::AcqRel);
        self.value.lock().insert(id, tx);
        id
    }

    pub fn invoke(&self, id: u32, msg: LHMFrame) {
        if let Some(tx) = self.value.lock().remove(&id) {
            _ = tx.send(msg);
        }
    }

    pub fn remove(&self, id: u32) {
        self.value.lock().remove(&id);
    }
}

/// Client for accessing the LHM service pipe
pub struct LHMClient;

impl LHMClient {
    /// Connect to the LHM service
    pub async fn connect() -> std::io::Result<LHMClientHandle> {
        let subscriptions = Arc::new(Subscriptions::default());
        let (future, rx, tx) = PipeFuture::connect().await?;

        spawn(async move {
            if let Err(err) = future.await {
                // TODO:
                eprintln!("{err}")
            }
        });

        spawn({
            let subscriptions = subscriptions.clone();
            let mut rx = rx;

            async move {
                while let Some(frame) = rx.recv().await {
                    subscriptions.invoke(frame.id, frame);
                }
            }
        });

        let handle = LHMClientHandle { subscriptions, tx };

        Ok(handle)
    }
}
