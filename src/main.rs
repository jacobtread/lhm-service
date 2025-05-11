use anyhow::Context;
use clap::{Parser, Subcommand};
use num_enum::TryFromPrimitive;

#[macro_use]
extern crate dlopen_derive;

mod actor;
mod ffi;
mod service;

pub use ffi::{Bridge, Computer};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Copy, TryFromPrimitive, Serialize)]
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
#[derive(Debug, Clone, Copy, TryFromPrimitive, Serialize)]
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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create the service (Will fail if the service is already created)
    Create,
    /// Start the service
    Start,
    /// Stop the service
    Stop,
    /// Restart the service
    Restart,
    /// Delete the service
    Delete,
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return match command {
            Commands::Create => service::create_service(),
            Commands::Start => service::start_service(),
            Commands::Stop => service::stop_service(),
            Commands::Restart => service::restart_service(),
            Commands::Delete => service::delete_service(),
        };
    }

    windows_service::service_dispatcher::start(service::SERVICE_NAME, ffi_service_main)
        .context("failed to start service")?;

    Ok(())
}

#[doc = r" Static callback used by the system to bootstrap the service."]
#[doc = r" Do not call it directly."]
extern "system" fn ffi_service_main(num_service_arguments: u32, service_arguments: *mut *mut u16) {
    let arguments = unsafe {
        windows_service::service_dispatcher::parse_service_arguments(
            num_service_arguments,
            service_arguments,
        )
    };
    service::service_main(arguments);
}
