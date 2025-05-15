//! # Windows Service
//!
//! Windows specific behavior for running OGuard as a Windows service, enables
//! the required behavior for listening to shutdown events from the service manager,
//! reporting the service state.
//!
//! Also includes logic for managing the service (Adding, Removing, and Restarting the service)

use anyhow::Context;
use std::env;
use std::ffi::OsString;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler;
use windows_service::service_control_handler::ServiceControlHandlerResult;

/// Name of the windows service
pub const SERVICE_NAME: &str = "LibreHardwareMonitorService";

/// Type of the service
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

fn main() -> anyhow::Result<()> {
    windows_service::service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .context("failed to start service")?;

    Ok(())
}

#[doc = r" Static callback used by the system to bootstrap the service."]
#[doc = r" Do not call it directly."]
extern "system" fn ffi_service_main(num_service_arguments: u32, service_arguments: *mut *mut u16) {
    let arguments = unsafe {
        windows_service::service_dispatcher::parse_service_arguments(
            num_service_arguments,
            service_arguments,
        )
    };

    service_main(arguments);
}

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
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let local_set = LocalSet::new();
    local_set.spawn_local(lhm_server::run_server());

    // Block waiting for shutdown
    runtime.block_on(local_set.run_until(async move {
        _ = shutdown_rx.recv().await;
    }));

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
