use actor::{ComputerActor, ComputerActorHandle, ComputerActorMessage};
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
        Err(_cause) => return,
    };

    let pipe = Framed::new(stream, LHMFrameCodec::default());
    let (future, mut rx, tx) = PipeFuture::new(pipe);

    // Spawn task to handle the pipe itself
    spawn(future);

    // Spawn task to handle messages from the pipe
    spawn(async move {
        while let Some(frame) = rx.recv().await {
            let handle = handle.clone();
            let tx = tx.clone();

            spawn(handle_frame(frame, handle, tx));
        }
    });
}

async fn handle_frame(frame: LHMFrame, handle: ComputerActorHandle, tx: PipeTx) {
    let request = rmp_serde::from_slice(&frame.body);
    let response = match request {
        // Handle the request
        Ok(request) => handle_request(request, handle)
            .await
            .unwrap_or_else(|err| err),
        // Failed to parse the request
        Err(err) => {
            let error = err.to_string();
            PipeResponse::Error { error }
        }
    };

    let response_bytes = match rmp_serde::to_vec(&response) {
        Ok(value) => value,
        Err(_cause) => {
            // Nothing we can do here
            return;
        }
    };

    _ = tx.send(LHMFrame {
        id: frame.id,
        body: Bytes::from(response_bytes),
    });
}

async fn handle_request(
    request: PipeRequest,
    handle: ComputerActorHandle,
) -> Result<PipeResponse, PipeResponse> {
    let (tx, rx) = oneshot::channel();

    handle
        .tx
        .send(ComputerActorMessage { request, tx })
        .map_err(|_| PipeResponse::Error {
            error: "request actor is closed".to_string(),
        })?;

    let response = rx.await.map_err(|_| PipeResponse::Error {
        error: "request actor is closed".to_string(),
    })?;

    Ok(response)
}
