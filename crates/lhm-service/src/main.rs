use anyhow::Context;

mod actor;
mod server;
mod service;

fn main() -> anyhow::Result<()> {
    windows_service::service_dispatcher::start(service::SERVICE_NAME, ffi_service_main)
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
    service::service_main(arguments);
}
