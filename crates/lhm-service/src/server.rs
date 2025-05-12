use crate::actor::ComputerActor;
use interprocess::os::windows::named_pipe::tokio::{DuplexPipeStream, PipeListenerOptionsExt};
use interprocess::os::windows::named_pipe::{PipeListenerOptions, pipe_mode};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use lhm_shared::PipeResponse;
use lhm_shared::{PIPE_NAME, PipeRequest};
use lhm_sys::Bridge;
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::spawn_local;
use widestring::U16CString;

pub async fn run_server() -> std::io::Result<()> {
    let bridge = Bridge::load()
        // Handle load error
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;

    let listener = PipeListenerOptions::new()
        .mode(interprocess::os::windows::named_pipe::PipeMode::Bytes)
        .security_descriptor(Some(SecurityDescriptor::deserialize(
            U16CString::from_str_truncate("D:(A;;GA;;;WD)").as_ucstr(),
        )?))
        .path(PIPE_NAME)
        .create_tokio_duplex::<pipe_mode::Bytes>()?;

    loop {
        let stream = listener.accept().await?;
        spawn_local(handle_pipe_stream(bridge.clone(), stream));
    }
}

pub async fn handle_pipe_stream(bridge: Bridge, mut stream: DuplexPipeStream<pipe_mode::Bytes>) {
    // Initialize an actor
    let handle = ComputerActor::create(bridge, Default::default());

    loop {
        let request: PipeRequest = match recv_message(&mut stream).await {
            Ok(value) => value,
            Err(_) => return,
        };

        match request {
            PipeRequest::Update => {
                if handle.update().await.is_err() {
                    return;
                }
            }
            PipeRequest::GetHardware => {
                let hardware = match handle.get_hardware().await {
                    Ok(value) => value,
                    Err(_) => return,
                };
                let response = PipeResponse::Hardware { hardware };

                if send_message(&mut stream, response).await.is_err() {
                    return;
                }
            }
            PipeRequest::SetOptions { options } => {
                if handle.set_options(options).await.is_err() {
                    return;
                }
            }
        }
    }
}

async fn send_message(
    stream: &mut DuplexPipeStream<pipe_mode::Bytes>,
    request: PipeResponse,
) -> std::io::Result<()> {
    let data_bytes =
        serde_json::to_vec(&request).map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
    let length = data_bytes.len() as u32;
    let length_bytes = length.to_be_bytes();

    // Write the length
    stream.write_all(&length_bytes).await?;
    // Write the actual message
    stream.write_all(&data_bytes).await?;

    // Flush the whole message
    stream.flush().await?;

    Ok(())
}

async fn recv_message(
    stream: &mut DuplexPipeStream<pipe_mode::Bytes>,
) -> std::io::Result<PipeRequest> {
    let mut len_buffer = [0u8; 4];

    // Read the length of the payload
    stream.read_exact(&mut len_buffer).await?;
    let length = u32::from_be_bytes(len_buffer) as usize;

    // Read the entire payload
    let mut data_buffer = vec![0u8; length];
    stream.read_exact(&mut data_buffer).await?;

    let response: PipeRequest = serde_json::from_slice(&data_buffer)
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
    Ok(response)
}
