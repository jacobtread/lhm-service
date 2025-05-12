use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PipeRequest {
    Update,
    GetHardware,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PipeResponse {
    Hardware { hardware: Vec<Hardware> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    /// Name of the hardware
    pub name: String,

    /// Type of hardware
    pub ty: HardwareType,

    /// Children for the hardware
    pub children: Vec<Hardware>,

    /// Sensors attached to the hardware
    pub sensors: Vec<Sensor>,
}

/// Instance of a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
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
    Debug, Clone, Copy, TryFromPrimitive, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u32)]
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
}

/// Types of sensors
#[derive(
    Debug, Clone, Copy, TryFromPrimitive, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u32)]
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
}
