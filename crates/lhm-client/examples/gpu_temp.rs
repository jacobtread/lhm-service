use lhm_client::{ComputerOptions, HardwareType, LHMClient, SensorType};
use lhm_shared::Hardware;
use tokio::try_join;

#[tokio::main]
async fn main() {
    let client = LHMClient::connect().await.unwrap();

    println!("Connected to client");

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

    println!("Set options");

    client.update_all().await.unwrap();

    println!("Updated hardware");

    // Query all GPU hardware
    let (gpu_nvidia, gpu_amd, gpu_intel) = try_join!(
        client.query_hardware(None, Some(HardwareType::GpuNvidia)),
        client.query_hardware(None, Some(HardwareType::GpuAmd)),
        client.query_hardware(None, Some(HardwareType::GpuIntel)),
    )
    .unwrap();

    // Collect all hardware into a list
    let gpus: Vec<Hardware> = gpu_nvidia
        .into_iter()
        .chain(gpu_amd.into_iter())
        .chain(gpu_intel.into_iter())
        .collect();

    for gpu in gpus {
        // Request all GPU temperature sensors
        let gpu_temps = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Temperature))
            .await
            .unwrap();

        // Find the package temperature
        let temp_sensor = gpu_temps
            .iter()
            .find(|sensor| sensor.name.eq("GPU Core"))
            .expect("Missing cpu temp sensor");

        println!("GPU {} is {}°C", gpu.name, temp_sensor.value);
    }
}
