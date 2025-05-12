use interprocess::os::windows::named_pipe::{pipe_mode, tokio::DuplexPipeStream};
use lhm_shared::{Hardware, HardwareType, PIPE_NAME, PipeRequest, PipeResponse, SensorType};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    //}

    let mut conn = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(PIPE_NAME)
        .await
        .unwrap();

    send_message(&mut conn, PipeRequest::Update).await;
    send_message(&mut conn, PipeRequest::GetHardware).await;
    let msg = recv_message(&mut conn).await;

    let hardware = match msg {
        PipeResponse::Hardware { hardware } => hardware,
    };

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

    println!("CPU is {temp}°C")
}

async fn send_message(stream: &mut DuplexPipeStream<pipe_mode::Bytes>, request: PipeRequest) {
    let data_bytes = serde_json::to_vec(&request).unwrap();
    let length = data_bytes.len() as u32;
    let length_bytes = length.to_be_bytes();

    // Write the length
    stream.write_all(&length_bytes).await.unwrap();
    // Write the actual message
    stream.write_all(&data_bytes).await.unwrap();

    // Flush the whole message
    stream.flush().await.unwrap();
}

async fn recv_message(stream: &mut DuplexPipeStream<pipe_mode::Bytes>) -> PipeResponse {
    let mut len_buffer = [0u8; 4];

    // Read the length of the payload
    stream.read_exact(&mut len_buffer).await.unwrap();
    let length = u32::from_be_bytes(len_buffer) as usize;

    // Read the entire payload
    let mut data_buffer = vec![0u8; length];
    stream.read_exact(&mut data_buffer).await.unwrap();

    let response: PipeResponse = serde_json::from_slice(&data_buffer).unwrap();
    response
}
