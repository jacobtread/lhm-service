use windows_service::{
    service::ServiceAccess,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

/// Name of the windows service
pub const SERVICE_NAME: &str = "LibreHardwareMonitorService";

/// Check if the LibreHardware Monitor service is installed
pub fn is_service_installed() -> Result<bool, windows_service::Error> {
    // Open the Service Control Manager with CONNECT access
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    // Try to open the service
    match manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS) {
        Ok(_service) => Ok(true),

        // ERROR_SERVICE_DOES_NOT_EXIST
        Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1060) => {
            Ok(false)
        }

        // Other errors (like access denied)
        Err(e) => Err(e),
    }
}
