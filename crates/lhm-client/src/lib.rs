use interprocess::os::windows::named_pipe::{pipe_mode, tokio::DuplexPipeStream};
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use lhm_shared::*;

pub struct LHMClient {
    pipe: DuplexPipeStream<pipe_mode::Bytes>,
}

impl LHMClient {
    pub async fn connect() -> std::io::Result<Self> {
        let pipe = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(PIPE_NAME).await?;
        Ok(Self { pipe })
    }

    async fn send(&mut self, request: PipeRequest) -> std::io::Result<()> {
        let data_bytes = rmp_serde::to_vec(&request)
            .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
        let length = data_bytes.len() as u32;
        let length_bytes = length.to_be_bytes();

        // Write the length
        self.pipe.write_all(&length_bytes).await?;
        // Write the actual message
        self.pipe.write_all(&data_bytes).await?;

        // Flush the whole message
        self.pipe.flush().await?;

        Ok(())
    }

    async fn recv(&mut self) -> std::io::Result<PipeResponse> {
        let mut len_buffer = [0u8; 4];

        // Read the length of the payload
        self.pipe.read_exact(&mut len_buffer).await?;
        let length = u32::from_be_bytes(len_buffer) as usize;

        // Read the entire payload
        let mut data_buffer = vec![0u8; length];
        self.pipe.read_exact(&mut data_buffer).await?;

        let response: PipeResponse = rmp_serde::from_slice(&data_buffer)
            .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
        Ok(response)
    }

    pub async fn update(&mut self) -> std::io::Result<()> {
        self.send(PipeRequest::Update).await?;
        match self.recv().await? {
            PipeResponse::Updated => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    pub async fn set_options(&mut self, options: ComputerOptions) -> std::io::Result<()> {
        self.send(PipeRequest::SetOptions { options }).await?;
        match self.recv().await? {
            PipeResponse::UpdatedOptions => Ok(()),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }

    pub async fn get_hardware(&mut self) -> std::io::Result<Vec<Hardware>> {
        self.send(PipeRequest::GetHardware).await?;
        match self.recv().await? {
            PipeResponse::Hardware { hardware } => Ok(hardware),
            PipeResponse::Error { error } => Err(std::io::Error::new(ErrorKind::Other, error)),
            _ => Err(std::io::Error::new(ErrorKind::Other, "unexpected message")),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ComputerOptions, Hardware, HardwareType, LHMClient, SensorType};

    #[tokio::test]
    #[ignore = "Requires the service to be running"]
    async fn main() {
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
    }
}
