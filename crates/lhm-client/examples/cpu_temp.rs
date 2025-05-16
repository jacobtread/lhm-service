use lhm_client::{ComputerOptions, HardwareType, LHMClient, SensorType};
use std::time::Duration;
use tokio::time::sleep;

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

    // Request all CPU hardware
    let cpu_list = client
        .query_hardware(None, Some(HardwareType::Cpu))
        .await
        .unwrap();

    println!("Querying hardware");

    for cpu in cpu_list {
        // Request all CPU temperature sensors
        let cpu_temps = client
            .query_sensors(Some(cpu.identifier.clone()), Some(SensorType::Temperature))
            .await
            .unwrap();

        dbg!(&cpu_temps);

        // Find the package temperature
        let temp_sensor = cpu_temps
            .iter()
            .find(|sensor| sensor.name.eq("CPU Package"))
            .expect("Missing cpu temp sensor");

        println!("CPU is initially {}°C", temp_sensor.value);

        for _ in 0..5 {
            // Get the current sensor value
            let value = client
                .get_sensor_value_by_idx(temp_sensor.index, true)
                .await
                .unwrap()
                .expect("cpu temp sensor is now unavailable");

            println!("CPU is now {}°C", value);
            sleep(Duration::from_secs(1)).await;
        }
    }
}
