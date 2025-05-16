use lhm_shared::{HardwareType, SensorType};
use std::collections::HashMap;

/// Cache holding the currently loaded hardware and sensors
/// in a fashion that is easily queryable with all sensors
/// and children resolved
#[derive(Default)]
pub struct HardwareCache {
    /// Flat collection of hardware (Includes all sub-hardware)
    hardware: Vec<HardwareEntry>,

    /// Collection of all sensors for all hardware
    sensors: Vec<SensorEntry>,

    /// Index cache for looking up hardware by ID
    hardware_lookup: HashMap<String, usize>,

    /// Index cache for looking up sensors by ID
    sensor_lookup: HashMap<String, usize>,
}

/// Hardware entry within the cache
pub struct HardwareEntry {
    /// Index of the parent entry within the cache
    /// if the entry is sub-hardware
    parent_index: Option<usize>,
    /// The hardware item itself
    hardware: lhm_sys::Hardware,
}

/// Sensor entry within the cache
pub struct SensorEntry {
    /// Index of the parent [HardwareEntry] in the cache
    parent_index: usize,
    /// The sensor item itself
    sensor: lhm_sys::Sensor,
}

impl HardwareCache {
    pub fn init(&mut self, hardware: Vec<lhm_sys::Hardware>) {
        self.clear();
        self.populate(hardware, None);
    }

    fn populate(&mut self, hardware: Vec<lhm_sys::Hardware>, parent_index: Option<usize>) {
        for item in hardware {
            let identifier = item.identifier();
            let children = item.get_children();
            let sensors = item.sensors();
            let hardware_index = self.hardware.len();

            self.hardware.push(HardwareEntry {
                parent_index,
                hardware: item,
            });
            self.hardware_lookup.insert(identifier, hardware_index);

            // Populate the sensors
            for sensor in sensors {
                let identifier = sensor.identifier();
                let sensor_index = self.sensors.len();

                self.sensors.push(SensorEntry {
                    parent_index: hardware_index,
                    sensor,
                });
                self.sensor_lookup.insert(identifier, sensor_index);
            }

            // Populate the cache with children
            self.populate(children, Some(hardware_index));
        }
    }

    pub fn get_hardware_by_id(&self, identifier: &str) -> Option<(usize, &lhm_sys::Hardware)> {
        let index = *self.hardware_lookup.get(identifier)?;
        let hardware = &self.hardware[index];
        Some((index, &hardware.hardware))
    }

    pub fn get_sensor_by_id(&self, identifier: &str) -> Option<(usize, &lhm_sys::Sensor)> {
        let index = *self.sensor_lookup.get(identifier)?;
        let sensor = &self.sensors[index];
        Some((index, &sensor.sensor))
    }

    pub fn get_hardware_by_id_mut(&mut self, identifier: &str) -> Option<&mut lhm_sys::Hardware> {
        let index = *self.hardware_lookup.get(identifier)?;
        let hardware = &mut self.hardware[index];
        Some(&mut hardware.hardware)
    }

    pub fn get_hardware_by_idx_mut(&mut self, index: usize) -> Option<&mut lhm_sys::Hardware> {
        self.hardware
            .get_mut(index)
            .map(|hardware| &mut hardware.hardware)
    }

    pub fn get_sensor_by_id_mut(&mut self, identifier: &str) -> Option<&mut lhm_sys::Sensor> {
        let index = *self.sensor_lookup.get(identifier)?;
        let sensor = &mut self.sensors[index];
        Some(&mut sensor.sensor)
    }

    pub fn get_sensor_by_idx_mut(&mut self, index: usize) -> Option<&mut lhm_sys::Sensor> {
        self.sensors.get_mut(index).map(|sensor| &mut sensor.sensor)
    }

    /// Creates an iterator that will iterate a filtered subset of the hardware
    /// optionally where the type or parent_id matches
    pub fn query_hardware_iter(
        &self,
        parent_index: Option<Option<usize>>,
        ty: Option<HardwareType>,
    ) -> impl Iterator<Item = (usize, &lhm_sys::Hardware)> + '_ {
        let ty_value: Option<i32> = ty.map(|value| value.into());

        self.hardware
            .iter()
            .enumerate()
            .filter(move |(_, hardware)| {
                // Filter by type
                if ty_value
                    .as_ref()
                    .is_some_and(|ty_value| hardware.hardware.get_type().ne(ty_value))
                {
                    return false;
                }

                // Filter by parent
                if parent_index.is_some_and(|parent_index| parent_index == hardware.parent_index) {
                    return false;
                }

                true
            })
            .map(|(index, entry)| (index, &entry.hardware))
    }

    pub fn query_sensors(
        &self,
        parent_index: Option<usize>,
        ty: Option<SensorType>,
    ) -> impl Iterator<Item = (usize, &lhm_sys::Sensor)> + '_ {
        let ty_value: Option<i32> = ty.map(|value| value.into());

        self.sensors
            .iter()
            .enumerate()
            .filter(move |(_, sensor)| {
                // Filter by type
                if ty_value
                    .as_ref()
                    .is_some_and(|ty_value| sensor.sensor.get_type().ne(ty_value))
                {
                    return false;
                }

                // Filter by parent
                if parent_index
                    .is_some_and(|parent_index: usize| sensor.parent_index != parent_index)
                {
                    return false;
                }

                true
            })
            .map(|(index, entry)| (index, &entry.sensor))
    }

    /// Empty the cache
    pub fn clear(&mut self) {
        self.hardware.clear();
        self.sensors.clear();
        self.hardware_lookup.clear();
        self.sensor_lookup.clear();
    }
}
