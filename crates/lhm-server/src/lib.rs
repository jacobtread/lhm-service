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
use tokio::spawn;
use tokio_util::{bytes::Bytes, codec::Framed};
use widestring::U16CString;
mod pipe;

mod actor;

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
    let handle = ComputerActor::create();
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
    match request {
        PipeRequest::UpdateAll => {
            handle.update_all().await?;
            Ok(PipeResponse::Success)
        }
        PipeRequest::SetOptions { options } => {
            handle.set_options(options).await?;
            Ok(PipeResponse::Success)
        }
        PipeRequest::GetHardwareById { id } => {
            let hardware = handle.get_hardware_by_id(id).await?;
            Ok(PipeResponse::Hardware { hardware })
        }
        PipeRequest::QueryHardware { parent_id, ty } => {
            let hardware = handle.get_hardware(parent_id, ty).await?;
            Ok(PipeResponse::Hardwares { hardware })
        }
        PipeRequest::UpdateHardwareById { id } => {
            handle.update_hardware_by_id(id).await?;
            Ok(PipeResponse::Success)
        }
        PipeRequest::GetSensorById { id } => {
            let sensor = handle.get_sensor_by_id(id).await?;
            Ok(PipeResponse::Sensor { sensor })
        }
        PipeRequest::GetSensorValueById { id, update } => {
            let value = handle.get_sensor_value_by_id(id, update).await?;
            Ok(PipeResponse::SensorValue { value })
        }
        PipeRequest::QuerySensors { parent_id, ty } => {
            let sensors = handle.get_sensors(parent_id, ty).await?;
            Ok(PipeResponse::Sensors { sensors })
        }
        PipeRequest::UpdateSensorById { id } => {
            handle.update_sensor_by_id(id).await?;
            Ok(PipeResponse::Success)
        }
        PipeRequest::UpdateHardwareByIndex { idx } => {
            handle.update_hardware_by_idx(idx).await?;
            Ok(PipeResponse::Success)
        }
        PipeRequest::GetSensorValueByIndex { idx, update } => {
            let value = handle.get_sensor_value_by_idx(idx, update).await?;
            Ok(PipeResponse::SensorValue { value })
        }
        PipeRequest::UpdateSensorByIndex { idx } => {
            handle.update_sensor_by_idx(idx).await?;
            Ok(PipeResponse::Success)
        }
    }
}
