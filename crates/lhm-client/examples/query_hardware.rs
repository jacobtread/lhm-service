use lhm_client::{ComputerOptions, LHMClient};

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
    let hardware = client.query_hardware(None, None).await.unwrap();
    let sensors = client.query_sensors(None, None).await.unwrap();
    dbg!(hardware);
    dbg!(sensors);
}
