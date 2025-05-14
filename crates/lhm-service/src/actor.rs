use std::collections::HashMap;

use lhm_shared::{ComputerOptions, Hardware, HardwareType, Sensor, SensorType};
use tokio::sync::{mpsc, oneshot};

use lhm_sys::{Computer, SharedApi};

#[derive(Clone)]
pub struct ComputerActorHandle {
    tx: mpsc::UnboundedSender<ComputerActorMessage>,
}

impl ComputerActorHandle {
    pub async fn set_options(&self, options: ComputerOptions) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::SetOptions { options, tx })?;
        rx.await.map_err(|err| err.into())
    }

    pub async fn update_all(&self) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::UpdateAll { tx })?;
        rx.await.map_err(|err| err.into())
    }

    pub async fn get_hardware_by_id(&self, identifier: String) -> anyhow::Result<Option<Hardware>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::GetHardwareById { id: identifier, tx })?;
        let value = rx.await?;
        Ok(value)
    }

    pub async fn get_hardware(
        &self,
        parent_identifier: Option<String>,
        ty: Option<HardwareType>,
    ) -> anyhow::Result<Vec<Hardware>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::QueryHardware {
            parent_id: parent_identifier,
            ty,
            tx,
        })?;
        let value = rx.await?;
        Ok(value)
    }

    pub async fn update_hardware_by_id(&self, identifier: String) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::UpdateHardwareById { id: identifier, tx })?;
        rx.await?;
        Ok(())
    }

    pub async fn update_hardware_by_idx(&self, idx: usize) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::UpdateHardwareByIndex { idx, tx })?;
        rx.await?;
        Ok(())
    }

    pub async fn get_sensor_by_id(&self, identifier: String) -> anyhow::Result<Option<Sensor>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::GetSensorById { id: identifier, tx })?;
        let value = rx.await?;
        Ok(value)
    }

    pub async fn get_sensor_value_by_id(
        &self,
        identifier: String,
        update: bool,
    ) -> anyhow::Result<Option<f32>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::GetSensorValueById {
            id: identifier,
            update,
            tx,
        })?;
        let value = rx.await?;
        Ok(value)
    }

    pub async fn get_sensor_value_by_idx(
        &self,
        idx: usize,
        update: bool,
    ) -> anyhow::Result<Option<f32>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::GetSensorValueByIndex { idx, update, tx })?;
        let value = rx.await?;
        Ok(value)
    }

    pub async fn get_sensors(
        &self,
        parent_identifier: Option<String>,
        ty: Option<SensorType>,
    ) -> anyhow::Result<Vec<Sensor>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ComputerActorMessage::QuerySensors {
            parent_id: parent_identifier,
            ty,
            tx,
        })?;
        let value = rx.await?;
        Ok(value)
    }

    pub async fn update_sensor_by_id(&self, identifier: String) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::UpdateSensorById { id: identifier, tx })?;
        rx.await?;
        Ok(())
    }

    pub async fn update_sensor_by_idx(&self, idx: usize) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ComputerActorMessage::UpdateSensorByIndex { idx, tx })?;
        rx.await?;
        Ok(())
    }
}

pub enum ComputerActorMessage {
    /// Update the options for the computer
    SetOptions {
        options: ComputerOptions,
        tx: oneshot::Sender<()>,
    },

    /// Update all hardware and sensors, will populate the cache
    UpdateAll {
        /// Sender for the completion notifications
        tx: oneshot::Sender<()>,
    },

    /// Get a specific hardware by ID
    GetHardwareById {
        /// Identifier of the hardware
        id: String,

        /// Sender for sending back the result
        tx: oneshot::Sender<Option<Hardware>>,
    },

    /// Get hardware
    QueryHardware {
        /// Identifier of the parent hardware if fetching children
        /// for a hardware
        parent_id: Option<String>,

        /// Type of hardware to get
        ty: Option<HardwareType>,

        /// Sender for sending the list of sensors back
        tx: oneshot::Sender<Vec<Hardware>>,
    },

    /// Update a specific hardware item
    UpdateHardwareById {
        /// Identifier of the hardware
        id: String,

        /// Sender to notify after the update
        tx: oneshot::Sender<()>,
    },

    /// Update a specific hardware item by index
    UpdateHardwareByIndex {
        /// Index of the hardware
        idx: usize,

        /// Sender to notify after the update
        tx: oneshot::Sender<()>,
    },

    /// Get a specific sensor by ID
    GetSensorById {
        /// Identifier of the sensor
        id: String,

        /// Sender for sending back the result
        tx: oneshot::Sender<Option<Sensor>>,
    },

    /// Get a specific sensor value by ID
    GetSensorValueById {
        /// Identifier of the sensor
        id: String,

        /// Whether to update the value before loading it
        update: bool,

        /// Sender for sending back the result
        tx: oneshot::Sender<Option<f32>>,
    },
    /// Get a specific sensor value by index
    GetSensorValueByIndex {
        /// Index of the sensor
        idx: usize,

        /// Whether to update the value before loading it
        update: bool,

        /// Sender for sending back the result
        tx: oneshot::Sender<Option<f32>>,
    },

    /// Get sensors
    QuerySensors {
        /// Optional identifier for a required parent hardware item. When set
        /// the hardware parent for the sensor must have a matching identifier
        parent_id: Option<String>,

        /// Type of the sensor
        ty: Option<SensorType>,

        /// Sender for sending the list of sensors back
        tx: oneshot::Sender<Vec<Sensor>>,
    },

    /// Update a specific sensor item
    UpdateSensorById {
        /// Identifier of the sensor
        id: String,

        /// Sender to notify after the update
        tx: oneshot::Sender<()>,
    },

    /// Update a specific sensor item using its index
    UpdateSensorByIndex {
        /// Index of the sensor
        idx: usize,

        /// Sender to notify after the update
        tx: oneshot::Sender<()>,
    },
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
        let ty_value: Option<u32> = ty.map(|value| value.into());

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
        let ty_value: Option<u32> = ty.map(|value| value.into());

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
    pub fn create(bridge: SharedApi) -> ComputerActorHandle {
        let (tx, rx) = mpsc::unbounded_channel();

        std::thread::spawn(move || {
            // Run service
            let computer = match Computer::create(bridge) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("failed to start computer service: {err}");
                    return;
                }
            };

            let actor = ComputerActor {
                computer,
                rx,
                cache: Default::default(),
            };
            actor.run();
        });

        ComputerActorHandle { tx }
    }

    fn run(mut self) {
        while let Some(msg) = self.rx.blocking_recv() {
            match msg {
                ComputerActorMessage::UpdateAll { tx } => {
                    self.computer.update();

                    // Load the hardware and populate the cache
                    let hardware = self.computer.hardware();
                    self.cache.clear();
                    self.cache.populate(hardware, None);

                    _ = tx.send(())
                }
                ComputerActorMessage::SetOptions { options, tx } => {
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
                    _ = tx.send(());
                }
                ComputerActorMessage::QueryHardware {
                    parent_id: hardware_identifier,
                    ty,
                    tx,
                } => {
                    let hardware = self
                        .cache
                        .query_hardware(hardware_identifier, ty)
                        .into_iter()
                        .map(|(index, hardware)| Hardware {
                            index,
                            identifier: hardware.identifier(),
                            name: hardware.name(),
                            ty: HardwareType::from(hardware.get_type()),
                        })
                        .collect();

                    _ = tx.send(hardware);
                }
                ComputerActorMessage::GetHardwareById { id: identifier, tx } => {
                    let hardware =
                        self.cache
                            .get_hardware_by_id(&identifier)
                            .map(|(index, hardware)| Hardware {
                                index,
                                identifier: hardware.identifier(),
                                name: hardware.name(),
                                ty: HardwareType::from(hardware.get_type()),
                            });

                    _ = tx.send(hardware);
                }
                ComputerActorMessage::UpdateHardwareById { id: identifier, tx } => {
                    if let Some(hardware) = self.cache.get_hardware_by_id_mut(&identifier) {
                        hardware.update();
                    }

                    _ = tx.send(())
                }
                ComputerActorMessage::UpdateHardwareByIndex { idx, tx } => {
                    if let Some(hardware) = self.cache.hardware.get_mut(idx) {
                        hardware.update();
                    }

                    _ = tx.send(())
                }
                ComputerActorMessage::GetSensorById { id: identifier, tx } => {
                    let sensor = self
                        .cache
                        .get_sensor_by_id(&identifier)
                        .map(|(index, sensor)| Sensor {
                            index,
                            identifier: sensor.identifier(),
                            name: sensor.name(),
                            ty: SensorType::from(sensor.get_type()),
                            value: sensor.value(),
                        });

                    _ = tx.send(sensor);
                }
                ComputerActorMessage::GetSensorValueById {
                    id: identifier,
                    update,
                    tx,
                } => {
                    let sensor = self.cache.get_sensor_by_id_mut(&identifier).map(|sensor| {
                        if update {
                            sensor.update();
                        }

                        sensor.value()
                    });

                    _ = tx.send(sensor);
                }
                ComputerActorMessage::GetSensorValueByIndex { idx, update, tx } => {
                    let sensor = self.cache.sensors.get_mut(idx).map(|sensor| {
                        if update {
                            sensor.update();
                        }

                        sensor.value()
                    });

                    _ = tx.send(sensor);
                }
                ComputerActorMessage::QuerySensors {
                    parent_id: hardware_identifier,
                    ty,
                    tx,
                } => {
                    let sensors = self
                        .cache
                        .query_sensors(hardware_identifier, ty)
                        .into_iter()
                        .map(|(index, sensor)| Sensor {
                            index,
                            identifier: sensor.identifier(),
                            name: sensor.name(),
                            ty: SensorType::from(sensor.get_type()),
                            value: sensor.value(),
                        })
                        .collect();

                    _ = tx.send(sensors);
                }

                ComputerActorMessage::UpdateSensorById { id: identifier, tx } => {
                    if let Some(sensor) = self.cache.get_sensor_by_id_mut(&identifier) {
                        sensor.update();
                    }

                    _ = tx.send(())
                }
                ComputerActorMessage::UpdateSensorByIndex { idx, tx } => {
                    if let Some(sensor) = self.cache.sensors.get_mut(idx) {
                        sensor.update();
                    }

                    _ = tx.send(())
                }
            }
        }
    }
}
