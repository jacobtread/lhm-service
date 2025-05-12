# LHM Client

Client that can consume the LHM Service from user-land 

```rust 
use lhm_client::{ComputerOptions, Hardware, HardwareType, LHMClient, SensorType};

let mut client = LHMClient::connect().await.unwrap();

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

client.update().await.unwrap();

let hardware = client.get_hardware().await.unwrap();

let cpus: Vec<&Hardware> = hardware
    .iter()
    .filter(|value| matches!(value.ty, HardwareType::Cpu))
    .collect();

let cpu_temps = cpus
    .iter()
    .flat_map(|value| {
        value
            .sensors
            .iter()
            .filter(|value| matches!(value.ty, SensorType::Temperature))
    })
    .collect::<Vec<_>>();

let temp = cpu_temps
    .iter()
    .find(|sensor| sensor.name.eq("CPU Package"));

let temp = temp.map(|value| value.value).expect("Unknown CPU Temp");

println!("CPU is {temp}°C");
```