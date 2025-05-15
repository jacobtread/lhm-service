use crate::cache::HardwareCache;
use lhm_shared::{Hardware, HardwareType, PipeRequest, PipeResponse, Sensor, SensorType};
use lhm_sys::Computer;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct ComputerActorHandle {
    pub tx: mpsc::UnboundedSender<ComputerActorMessage>,
}

pub struct ComputerActorMessage {
    pub request: PipeRequest,
    pub tx: oneshot::Sender<PipeResponse>,
}

pub struct ComputerActor {
    computer: Computer,
    cache: HardwareCache,
}

impl ComputerActor {
    /// Create a new actor thread
    pub fn create() -> std::io::Result<ComputerActorHandle> {
        let computer = Computer::create()?;

        let (tx, rx) = mpsc::unbounded_channel();
        let actor = ComputerActor {
            computer,
            cache: Default::default(),
        };

        std::thread::spawn(move || {
            let mut rx = rx;
            let mut actor = actor;

            while let Some(ComputerActorMessage { request, tx }) = rx.blocking_recv() {
                let response = actor.handle_request(request);
                _ = tx.send(response);
            }
        });

        Ok(ComputerActorHandle { tx })
    }

    fn handle_request(&mut self, request: PipeRequest) -> PipeResponse {
        match request {
            PipeRequest::UpdateAll => {
                self.computer.update();

                // Load the hardware and populate the cache
                let hardware = self.computer.hardware();
                self.cache.init(hardware);
            }

            PipeRequest::SetOptions { options } => {
                self.computer.set_options(lhm_sys::ComputerOptions {
                    battery_enabled: options.battery_enabled,
                    controller_enabled: options.controller_enabled,
                    cpu_enabled: options.cpu_enabled,
                    gpu_enabled: options.gpu_enabled,
                    memory_enabled: options.memory_enabled,
                    motherboard_enabled: options.motherboard_enabled,
                    network_enabled: options.network_enabled,
                    psu_enabled: options.psu_enabled,
                    storage_enabled: options.storage_enabled,
                });
            }

            PipeRequest::QueryHardware { parent_id, ty } => {
                // Lookup the index for the parent item
                let parent_index = match parent_id.map(|id| self.cache.get_hardware_by_id(&id)) {
                    // Index of the parent item
                    Some(Some((index, _))) => Some(index),
                    // Parent hardware did not exist
                    Some(None) => {
                        return PipeResponse::Error {
                            error: "parent not found".to_string(),
                        };
                    }
                    // Not requesting a parent index
                    None => None,
                };

                let hardware = self
                    .cache
                    .query_hardware_iter(parent_index, ty)
                    .map(map_hardware)
                    .collect();

                return PipeResponse::Hardwares { hardware };
            }

            PipeRequest::QuerySensors { parent_id, ty } => {
                // Lookup the index for the parent item
                let parent_index = match parent_id.map(|id| self.cache.get_hardware_by_id(&id)) {
                    // Index of the parent item
                    Some(Some((index, _))) => Some(index),
                    // Parent hardware did not exist
                    Some(None) => {
                        return PipeResponse::Error {
                            error: "parent not found".to_string(),
                        };
                    }
                    // Not requesting a parent index
                    None => None,
                };

                let sensors = self
                    .cache
                    .query_sensors(parent_index, ty)
                    .map(map_sensor)
                    .collect();

                return PipeResponse::Sensors { sensors };
            }

            PipeRequest::GetHardwareById { id } => {
                let hardware = self.cache.get_hardware_by_id(&id).map(map_hardware);
                return PipeResponse::Hardware { hardware };
            }

            PipeRequest::GetSensorById { id } => {
                let sensor = self.cache.get_sensor_by_id(&id).map(map_sensor);
                return PipeResponse::Sensor { sensor };
            }

            PipeRequest::UpdateHardwareById { id } => {
                if let Some(hardware) = self.cache.get_hardware_by_id_mut(&id) {
                    hardware.update();
                }
            }

            PipeRequest::UpdateHardwareByIndex { idx } => {
                if let Some(hardware) = self.cache.get_hardware_by_idx_mut(idx) {
                    hardware.update();
                }
            }

            PipeRequest::UpdateSensorById { id } => {
                if let Some(sensor) = self.cache.get_sensor_by_id_mut(&id) {
                    sensor.update();
                }
            }

            PipeRequest::UpdateSensorByIndex { idx } => {
                if let Some(sensor) = self.cache.get_sensor_by_idx_mut(idx) {
                    sensor.update();
                }
            }

            PipeRequest::GetSensorValueById { id, update } => {
                let value = self.cache.get_sensor_by_id_mut(&id).map(|sensor| {
                    if update {
                        sensor.update();
                    }

                    sensor.value()
                });

                return PipeResponse::SensorValue { value };
            }

            PipeRequest::GetSensorValueByIndex { idx, update } => {
                let value = self.cache.get_sensor_by_idx_mut(idx).map(|sensor| {
                    if update {
                        sensor.update();
                    }

                    sensor.value()
                });

                return PipeResponse::SensorValue { value };
            }
        }

        PipeResponse::Success
    }
}

fn map_hardware((index, hardware): (usize, &lhm_sys::Hardware)) -> lhm_shared::Hardware {
    Hardware {
        index,
        identifier: hardware.identifier(),
        name: hardware.name(),
        ty: HardwareType::from(hardware.get_type()),
    }
}

fn map_sensor((index, sensor): (usize, &lhm_sys::Sensor)) -> lhm_shared::Sensor {
    Sensor {
        index,
        identifier: sensor.identifier(),
        name: sensor.name(),
        ty: SensorType::from(sensor.get_type()),
        value: sensor.value(),
    }
}
