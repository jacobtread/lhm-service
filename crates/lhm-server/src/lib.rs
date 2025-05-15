use crate::actor::{ComputerActor, ComputerActorHandle};
use actor::ComputerActorMessage;
use interprocess::os::windows::{
    named_pipe::{
        PipeListenerOptions, pipe_mode,
        tokio::{DuplexPipeStream, PipeListenerOptionsExt},
    },
    security_descriptor::SecurityDescriptor,
};
use lhm_shared::{
    PIPE_NAME, PipeRequest, PipeResponse,
    codec::{LHMFrame, LHMFrameCodec},
};
use pipe::{PipeFuture, PipeTx};
use tokio::{spawn, sync::oneshot};
use tokio_util::{bytes::Bytes, codec::Framed};
use widestring::U16CString;

mod actor;
mod cache;
mod pipe;

/// Run the server
pub async fn run_server() -> std::io::Result<()> {
    // Security descriptor that allows user-land programs to access the pipe
    let security_descriptor = SecurityDescriptor::deserialize(
        U16CString::from_str_truncate("D:(A;;GA;;;WD)").as_ucstr(),
    )?;

    // Create the pipe
    let listener = PipeListenerOptions::new()
        .mode(interprocess::os::windows::named_pipe::PipeMode::Bytes)
        .security_descriptor(Some(security_descriptor))
        .path(PIPE_NAME)
        .create_tokio_duplex::<pipe_mode::Bytes>()?;

    loop {
        let stream = listener.accept().await?;
        handle_pipe_stream(stream);
    }
}

pub type Pipe = Framed<DuplexPipeStream<pipe_mode::Bytes>, LHMFrameCodec>;

pub fn handle_pipe_stream(stream: DuplexPipeStream<pipe_mode::Bytes>) {
    // Initialize an actor
    let handle = match ComputerActor::create() {
        Ok(value) => value,
        Err(_cause) => {
            return;
        }
    };

    let pipe = Framed::new(stream, LHMFrameCodec::default());

    let (future, mut rx, tx) = PipeFuture::new(pipe);

    spawn(future);
    spawn(async move {
        while let Some(frame) = rx.recv().await {
            handle_frame(frame, &handle, &tx);
        }
    });
}

fn handle_frame(frame: LHMFrame, handle: &ComputerActorHandle, tx: &PipeTx) {
    let request = rmp_serde::from_slice(&frame.body)
        // Translate error type
        .map_err(|err| {
            let error = err.to_string();
            PipeResponse::Error { error }
        });

    let request: PipeRequest = match request {
        Ok(value) => value,
        Err(err) => {
            let frame = match serialize_frame(frame.id, err) {
                Ok(value) => value,
                Err(_cause) => {
                    // Nothing we can do here
                    return;
                }
            };

            _ = tx.send(frame);
            return;
        }
    };

    spawn(handle_request(
        frame.id,
        request,
        handle.clone(),
        tx.clone(),
    ));
}

async fn handle_request(id: u32, request: PipeRequest, handle: ComputerActorHandle, tx: PipeTx) {
    let response = handle_request_inner(request, handle)
        .await
        // Create a error response
        .unwrap_or_else(|err| {
            let error = err.to_string();
            PipeResponse::Error { error }
        });

    let frame = match serialize_frame(id, response) {
        Ok(value) => value,
        Err(_cause) => {
            // Nothing we can do here
            return;
        }
    };

    _ = tx.send(frame);
}

fn serialize_frame(id: u32, message: PipeResponse) -> Result<LHMFrame, rmp_serde::encode::Error> {
    let data_bytes = rmp_serde::to_vec(&message)?;
    Ok(LHMFrame {
        id,
        body: Bytes::from(data_bytes),
    })
}

pub async fn handle_request_inner(
    request: PipeRequest,
    handle: ComputerActorHandle,
) -> anyhow::Result<PipeResponse> {
    let (tx, rx) = oneshot::channel();
    handle.tx.send(ComputerActorMessage { request, tx })?;
    let response = rx.await?;
    Ok(response)
}
