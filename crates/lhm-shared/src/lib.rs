use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};

pub mod codec;

pub const PIPE_NAME: &str = r"\\.\pipe\LHMLibreHardwareMonitorService";

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ComputerOptions {
    pub battery_enabled: bool,
    pub controller_enabled: bool,
    pub cpu_enabled: bool,
    pub gpu_enabled: bool,
    pub memory_enabled: bool,
    pub motherboard_enabled: bool,
    pub network_enabled: bool,
    pub psu_enabled: bool,
    pub storage_enabled: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PipeRequest {
    SetOptions {
        options: ComputerOptions,
    },
    UpdateAll,
    GetHardwareById {
        id: String,
    },
    QueryHardware {
        parent_id: Option<String>,
        ty: Option<HardwareType>,
    },
    UpdateHardwareById {
        id: String,
    },
    UpdateHardwareByIndex {
        idx: usize,
    },
    GetSensorById {
        id: String,
    },
    GetSensorValueById {
        id: String,
        update: bool,
    },
    GetSensorValueByIndex {
        idx: usize,
        update: bool,
    },
    QuerySensors {
        parent_id: Option<String>,
        ty: Option<SensorType>,
    },
    UpdateSensorById {
        id: String,
    },
    UpdateSensorByIndex {
        idx: usize,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PipeResponse {
    Hardware { hardware: Option<Hardware> },
    Hardwares { hardware: Vec<Hardware> },

    Sensor { sensor: Option<Sensor> },
    SensorValue { value: Option<f32> },
    Sensors { sensors: Vec<Sensor> },

    Success,
    Error { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    /// Cache index of the hardware
    pub index: usize,

    /// Unique identifier for the hardware
    pub identifier: String,

    /// Name of the hardware
    pub name: String,

    /// Type of hardware
    pub ty: HardwareType,
}

/// Instance of a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    /// Cache index of the sensor
    pub index: usize,

    /// Unique identifier for the hardware
    pub identifier: String,

    /// Name of the sensor
    pub name: String,

    /// Type of the sensor
    pub ty: SensorType,

    /// Value of the sensor will be NaN when the sensor
    /// has no value
    pub value: f32,
}

/// Types of hardware
#[derive(
    Debug,
    Clone,
    Copy,
    FromPrimitive,
    IntoPrimitive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[repr(i32)]
pub enum HardwareType {
    Motherboard,
    SuperIO,
    Cpu,
    Memory,
    GpuNvidia,
    GpuAmd,
    GpuIntel,
    Storage,
    Network,
    Cooler,
    EmbeddedController,
    Psu,
    Battery,
    #[num_enum(catch_all)]
    Unknown(i32),
}

/// Types of sensors
#[derive(
    Debug,
    Clone,
    Copy,
    FromPrimitive,
    IntoPrimitive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[repr(i32)]
pub enum SensorType {
    Voltage,
    Current,
    Power,
    Clock,
    Temperature,
    Load,
    Frequency,
    Fan,
    Flow,
    Control,
    Level,
    Factor,
    Data,
    SmallData,
    Throughput,
    TimeSpan,
    Energy,
    Noise,
    Conductivity,
    Humidity,
    #[num_enum(catch_all)]
    Unknown(i32),
}
