use lhm_shared::{Hardware, HardwareType, PipeRequest, PipeResponse, Sensor, SensorType};
use lhm_sys::Computer;
use std::collections::HashMap;
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
    rx: mpsc::UnboundedReceiver<ComputerActorMessage>,
    cache: HardwareCache,
}

#[derive(Default)]
struct HardwareCache {
    /// Flat collection of hardware (Includes all sub-hardware)
    hardware: Vec<lhm_sys::Hardware>,

    /// Collection of all sensors for all hardware
    sensors: Vec<lhm_sys::Sensor>,

    /// Index cache for looking up hardware by ID
    hardware_lookup: HashMap<String, HardwareLookupIndex>,

    /// Index cache for looking up sensors by ID
    sensor_lookup: HashMap<String, SensorLookupIndex>,
}

pub struct HardwareLookupIndex {
    parent_index: Option<usize>,
    index: usize,
}

pub struct SensorLookupIndex {
    parent_index: usize,
    index: usize,
}

impl HardwareCache {
    pub fn populate(&mut self, hardware: Vec<lhm_sys::Hardware>, parent_index: Option<usize>) {
        for item in hardware {
            let identifier = item.identifier();
            let children = item.get_children();
            let sensors = item.sensors();
            let hardware_index = self.hardware.len();

            self.hardware.push(item);
            self.hardware_lookup.insert(
                identifier,
                HardwareLookupIndex {
                    parent_index,
                    index: hardware_index,
                },
            );

            // Populate the sensors
            for sensor in sensors {
                let identifier = sensor.identifier();
                let sensor_index = self.sensors.len();

                self.sensors.push(sensor);
                self.sensor_lookup.insert(
                    identifier,
                    SensorLookupIndex {
                        parent_index: hardware_index,
                        index: sensor_index,
                    },
                );
            }

            // Populate the cache with children
            self.populate(children, Some(hardware_index));
        }
    }

    pub fn get_hardware_by_id(&self, identifier: &str) -> Option<(usize, &lhm_sys::Hardware)> {
        let lookup_index = self.hardware_lookup.get(identifier)?;
        let index = lookup_index.index;
        let hardware = &self.hardware[index];
        Some((index, hardware))
    }

    pub fn get_sensor_by_id(&self, identifier: &str) -> Option<(usize, &lhm_sys::Sensor)> {
        let lookup_index = self.sensor_lookup.get(identifier)?;
        let index = lookup_index.index;
        let sensor = &self.sensors[index];
        Some((index, sensor))
    }

    pub fn get_hardware_by_id_mut(&mut self, identifier: &str) -> Option<&mut lhm_sys::Hardware> {
        let lookup_index = self.hardware_lookup.get(identifier)?;
        let index = lookup_index.index;
        let hardware = &mut self.hardware[index];
        Some(hardware)
    }

    pub fn get_sensor_by_id_mut(&mut self, identifier: &str) -> Option<&mut lhm_sys::Sensor> {
        let lookup_index = self.sensor_lookup.get(identifier)?;
        let index = lookup_index.index;
        let sensor = &mut self.sensors[index];
        Some(sensor)
    }

    pub fn query_hardware(
        &self,
        hardware_identifier: Option<String>,
        ty: Option<HardwareType>,
    ) -> Vec<(usize, &lhm_sys::Hardware)> {
        let ty_value: Option<i32> = ty.map(|value| value.into());

        // Lookup the index for the parent item
        let parent_index = match hardware_identifier {
            Some(value) => {
                match self.hardware_lookup.get(&value) {
                    Some(value) => Some(value.index),

                    // Parent hardware did not exist
                    None => return Vec::new(),
                }
            }

            None => None,
        };

        self.hardware
            .iter()
            .enumerate()
            .filter(|(_, hardware)| {
                // Filter by type
                if let Some(ty_value) = &ty_value {
                    if hardware.get_type().ne(ty_value) {
                        return false;
                    }
                }

                // Filter by parent
                if let Some(parent_index) = parent_index {
                    let identifier = hardware.identifier();
                    let index = match self.hardware_lookup.get(&identifier) {
                        Some(value) => value,
                        None => return false,
                    };

                    // Parent must match
                    if index.parent_index.is_none_or(|value| value == parent_index) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    pub fn query_sensors(
        &self,
        hardware_identifier: Option<String>,
        ty: Option<SensorType>,
    ) -> Vec<(usize, &lhm_sys::Sensor)> {
        let ty_value: Option<i32> = ty.map(|value| value.into());

        // Lookup the index for the parent item
        let parent_index = match hardware_identifier {
            Some(value) => {
                match self.hardware_lookup.get(&value) {
                    Some(value) => Some(value.index),

                    // Parent hardware did not exist
                    None => return Vec::new(),
                }
            }

            None => None,
        };

        self.sensors
            .iter()
            .enumerate()
            .filter(|(_, sensor)| {
                // Filter by type
                if let Some(ty_value) = &ty_value {
                    if sensor.get_type().ne(ty_value) {
                        return false;
                    }
                }

                // Filter by parent
                if let Some(parent_index) = parent_index {
                    let identifier = sensor.identifier();
                    let index = match self.sensor_lookup.get(&identifier) {
                        Some(value) => value,
                        None => return false,
                    };

                    // Parent must match
                    if index.parent_index != parent_index {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.hardware.clear();
        self.sensors.clear();
        self.hardware_lookup.clear();
        self.sensor_lookup.clear();
    }
}

impl ComputerActor {
    pub fn create() -> std::io::Result<ComputerActorHandle> {
        let (tx, rx) = mpsc::unbounded_channel();

        let computer = Computer::create()?;

        let actor = ComputerActor {
            computer,
            rx,
            cache: Default::default(),
        };

        std::thread::spawn(move || actor.run());

        Ok(ComputerActorHandle { tx })
    }

    fn run(mut self) {
        while let Some(ComputerActorMessage { request, tx }) = self.rx.blocking_recv() {
            let response = self.handle_request(request);
            _ = tx.send(response);
        }
    }

    fn handle_request(&mut self, request: PipeRequest) -> PipeResponse {
        match request {
            PipeRequest::UpdateAll => {
                self.computer.update();

                // Load the hardware and populate the cache
                let hardware = self.computer.hardware();
                self.cache.clear();
                self.cache.populate(hardware, None);

                PipeResponse::Success
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

                PipeResponse::Success
            }

            PipeRequest::QueryHardware { parent_id, ty } => {
                let hardware = self
                    .cache
                    .query_hardware(parent_id, ty)
                    .into_iter()
                    .map(|(index, hardware)| Hardware {
                        index,
                        identifier: hardware.identifier(),
                        name: hardware.name(),
                        ty: HardwareType::from(hardware.get_type()),
                    })
                    .collect();

                PipeResponse::Hardwares { hardware }
            }

            PipeRequest::GetHardwareById { id } => {
                let hardware =
                    self.cache
                        .get_hardware_by_id(&id)
                        .map(|(index, hardware)| Hardware {
                            index,
                            identifier: hardware.identifier(),
                            name: hardware.name(),
                            ty: HardwareType::from(hardware.get_type()),
                        });

                PipeResponse::Hardware { hardware }
            }

            PipeRequest::UpdateHardwareById { id } => {
                if let Some(hardware) = self.cache.get_hardware_by_id_mut(&id) {
                    hardware.update();
                }

                PipeResponse::Success
            }

            PipeRequest::UpdateHardwareByIndex { idx } => {
                if let Some(hardware) = self.cache.hardware.get_mut(idx) {
                    hardware.update();
                }

                PipeResponse::Success
            }

            PipeRequest::GetSensorById { id } => {
                let sensor = self
                    .cache
                    .get_sensor_by_id(&id)
                    .map(|(index, sensor)| Sensor {
                        index,
                        identifier: sensor.identifier(),
                        name: sensor.name(),
                        ty: SensorType::from(sensor.get_type()),
                        value: sensor.value(),
                    });

                PipeResponse::Sensor { sensor }
            }

            PipeRequest::GetSensorValueById { id, update } => {
                let value = self.cache.get_sensor_by_id_mut(&id).map(|sensor| {
                    if update {
                        sensor.update();
                    }

                    sensor.value()
                });

                PipeResponse::SensorValue { value }
            }

            PipeRequest::GetSensorValueByIndex { idx, update } => {
                let value = self.cache.sensors.get_mut(idx).map(|sensor| {
                    if update {
                        sensor.update();
                    }

                    sensor.value()
                });

                PipeResponse::SensorValue { value }
            }

            PipeRequest::QuerySensors { parent_id, ty } => {
                let sensors = self
                    .cache
                    .query_sensors(parent_id, ty)
                    .into_iter()
                    .map(|(index, sensor)| Sensor {
                        index,
                        identifier: sensor.identifier(),
                        name: sensor.name(),
                        ty: SensorType::from(sensor.get_type()),
                        value: sensor.value(),
                    })
                    .collect();

                PipeResponse::Sensors { sensors }
            }

            PipeRequest::UpdateSensorById { id } => {
                if let Some(sensor) = self.cache.get_sensor_by_id_mut(&id) {
                    sensor.update();
                }

                PipeResponse::Success
            }

            PipeRequest::UpdateSensorByIndex { idx } => {
                if let Some(sensor) = self.cache.sensors.get_mut(idx) {
                    sensor.update();
                }

                PipeResponse::Success
            }
        }
    }
}
