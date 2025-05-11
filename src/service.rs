//! # Windows Service
//!
//! Windows specific behavior for running OGuard as a Windows service, enables
//! the required behavior for listening to shutdown events from the service manager,
//! reporting the service state.
//!
//! Also includes logic for managing the service (Adding, Removing, and Restarting the service)

use anyhow::Context;
use interprocess::os::windows::named_pipe::tokio::{DuplexPipeStream, PipeListenerOptionsExt};
use interprocess::os::windows::named_pipe::{PipeListenerOptions, pipe_mode};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tracing::debug;
use widestring::U16CString;
use windows_service::service::{
    ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
    ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler;
use windows_service::service_control_handler::ServiceControlHandlerResult;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

use crate::Hardware;
use crate::actor::{ComputerActor, ComputerActorHandle};

/// Name of the windows service
pub const SERVICE_NAME: &str = "lhm-hardware-monitor-service";

/// Display name for the service
const SERVICE_DISPLAY_NAME: &str = "LHM Hardware Monitor";

/// Type of the service
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

// Service entrypoint
pub fn service_main(_arguments: Vec<OsString>) {
    setup_working_directory().expect("failed to setup working directory");

    if let Err(_err) = run_service() {}
}

/// Sets up the working directory for the service.
///
/// Services run with the working directory set to C:/Windows/System32 which
/// is not where we want to store and load our application files from. We
/// replace this with the executable path
fn setup_working_directory() -> anyhow::Result<()> {
    // Get the path to the executable
    let exe_path = env::current_exe().context("failed to get current executable path")?;

    // Get the directory containing the executable
    let exe_dir = exe_path
        .parent()
        .context("Failed to get parent directory")?;

    // Set the working directory to the executable's directory
    env::set_current_dir(exe_dir).context("failed to set current directory")?;

    // Set the working directory to the executable's directory
    env::set_current_dir(exe_dir).context("failed to set current directory")?;

    Ok(())
}

/// Restarts the windows service
pub fn restart_service() -> anyhow::Result<()> {
    stop_service()?;
    start_service()?;
    Ok(())
}

/// Creates a new windows service for the oguard exe using sc.exe
pub fn create_service() -> anyhow::Result<()> {
    debug!("creating service");

    // Get the path to the executable
    let executable_path = env::current_exe().context("failed to get current executable path")?;

    let manager =
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)
            .context("failed to access service manager")?;

    // Create the service
    manager
        .create_service(
            &ServiceInfo {
                name: OsString::from(SERVICE_NAME),
                display_name: OsString::from(SERVICE_DISPLAY_NAME),
                service_type: SERVICE_TYPE,
                start_type: ServiceStartType::AutoStart,
                error_control: ServiceErrorControl::Normal,
                executable_path,
                launch_arguments: vec![],
                dependencies: vec![],
                account_name: None, // run as System
                account_password: None,
            },
            ServiceAccess::QUERY_STATUS,
        )
        .context("failed to create service")?;

    Ok(())
}

/// Starts the windows service
pub fn start_service() -> anyhow::Result<()> {
    debug!("starting service");

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("failed to access service manager")?;

    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::START)
        .context("failed to open service")?;
    service
        .start::<&str>(&[])
        .context("failed to start service")?;

    Ok(())
}

/// Stops the windows service
pub fn stop_service() -> anyhow::Result<()> {
    debug!("stopping service");

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("failed to access service manager")?;

    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::STOP)
        .context("failed to open service")?;

    service.stop().context("failed to stop service")?;

    Ok(())
}

/// Deletes the windows service
pub fn delete_service() -> anyhow::Result<()> {
    debug!("deleting service");

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("failed to access service manager")?;

    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::DELETE)
        .context("failed to open service")?;

    service.delete().context("failed to delete service")?;

    Ok(())
}

/// Runs the service and handles service events
fn run_service() -> anyhow::Result<()> {
    // Create a channel to be able to poll a stop event from the service worker loop.
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                _ = shutdown_tx.try_send(());
                ServiceControlHandlerResult::NoError
            }

            // treat the UserEvent as a stop request
            ServiceControl::UserEvent(code) => {
                if code.to_raw() == 130 {
                    _ = shutdown_tx.try_send(());
                }
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Create the async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    runtime.spawn(run_server());

    // Block waiting for shutdown
    runtime.block_on(async move {
        _ = shutdown_rx.recv().await;
    });

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

async fn run_server() -> std::io::Result<()> {
    let handle = ComputerActor::create();

    let listener = PipeListenerOptions::new()
        .mode(interprocess::os::windows::named_pipe::PipeMode::Bytes)
        .security_descriptor(Some(SecurityDescriptor::deserialize(
            U16CString::from_str_truncate("D:(A;;GA;;;WD)").as_ucstr(),
        )?))
        .path(r"\\.\pipe\LHMLibreHardwareMonitorService")
        .create_tokio_duplex::<pipe_mode::Bytes>()?;

    loop {
        let stream = listener.accept().await?;
        let handle = handle.clone();
        tokio::spawn(handle_pipe_stream(handle, stream));
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum PipeRequest {
    Update,
    GetHardware,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum PipeResponse {
    Hardware { hardware: Vec<Hardware> },
}

pub async fn handle_pipe_stream(
    handle: ComputerActorHandle,
    mut stream: DuplexPipeStream<pipe_mode::Bytes>,
) {
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
