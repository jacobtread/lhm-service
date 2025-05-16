use lhm_client::{ComputerOptions, HardwareType, LHMClient, SensorType};

#[tokio::main]
async fn main() {
    println!("Connected to client");
    let client = LHMClient::connect().await.unwrap();

    // Set the computer options
    println!("Set options");
    client
        .set_options(ComputerOptions {
            controller_enabled: true,
            cpu_enabled: true,
            gpu_enabled: true,
            motherboard_enabled: true,
            battery_enabled: true,
            memory_enabled: true,
            network_enabled: true,
            psu_enabled: true,
            storage_enabled: true,
        })
        .await
        .unwrap();

    // Perform update to load hardware
    println!("Updated hardware");
    client.update_all().await.unwrap();

    // Request all hardware
    println!("Query hardware");
    let mut hardware = client
        .query_hardware(None, Some(HardwareType::Memory))
        .await
        .unwrap();

    let memory = hardware.first_mut().expect("missing memory hardware");

    println!("Query sensors");
    let sensors = client
        .query_sensors(Some(memory.identifier.clone()), None)
        .await
        .unwrap();

    let memory_used_sensor = sensors
        .iter()
        .find(|sensor| matches!(sensor.ty, SensorType::Data) && sensor.name.eq("Memory Used"))
        .expect("missing memory used");
    let memory_used_gb = memory_used_sensor.value;

    let memory_available_sensor = sensors
        .iter()
        .find(|sensor| matches!(sensor.ty, SensorType::Data) && sensor.name.eq("Memory Available"))
        .expect("missing memory available");
    let memory_available_gb = memory_available_sensor.value;

    let virt_memory_used_sensor = sensors
        .iter()
        .find(|sensor| {
            matches!(sensor.ty, SensorType::Data) && sensor.name.eq("Virtual Memory Used")
        })
        .expect("missing virtual memory used");
    let virt_memory_used_gb = virt_memory_used_sensor.value;

    let virt_memory_available_sensor = sensors
        .iter()
        .find(|sensor| {
            matches!(sensor.ty, SensorType::Data) && sensor.name.eq("Virtual Memory Available")
        })
        .expect("missing virtual memory available");
    let virt_memory_available_gb = virt_memory_available_sensor.value;

    println!("Memory Available: {memory_available_gb:.2}GB");
    println!("Memory Used: {memory_used_gb:.2}GB");

    println!("Virtual Memory Available: {virt_memory_available_gb:.2}GB");
    println!("Virtual Memory Used: {virt_memory_used_gb:.2}GB");
}
