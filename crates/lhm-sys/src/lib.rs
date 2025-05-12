//! # lhm-sys
//!
//! System library for working with Libre Hardware Monitor from rust, specifically creating a
//! computer instance, updating it and requesting the list of hardware and sensors from Rust
//!
//! Requires .NET SDK 8.0
//!
//! You can install this through winget using:
//! ```
//! winget install Microsoft.DotNet.SDK.8
//!```
//!

#[macro_use]
extern crate dlopen_derive;

mod ffi;

use std::sync::Arc;

use ffi::{ComputerInstance, SharedBridgeContainer, load_bridge_dll};
use lhm_shared::{ComputerOptions, Hardware};

/// Instance of the bridge library used for creating computer instances
#[derive(Clone)]
pub struct Bridge {
    inner: SharedBridgeContainer,
}

impl Bridge {
    pub fn load() -> Result<Bridge, dlopen::Error> {
        let container = load_bridge_dll()?;
        Ok(Self {
            inner: Arc::new(container),
        })
    }
}

/// Instance of a computer, can be used to request and update the list
/// of devices. Must call [Computer::update] at least once for hardware
/// to be available
pub struct Computer {
    instance: ComputerInstance,
}

impl Computer {
    pub fn create(bridge: &Bridge, options: ComputerOptions) -> Self {
        let instance = ComputerInstance::create(bridge.inner.clone(), options.into());
        Self { instance }
    }

    pub fn update(&mut self) {
        self.instance.update();
    }

    pub fn update_options(&mut self, options: ComputerOptions) {
        self.instance.update_options(options.into());
    }

    pub fn get_hardware(&mut self) -> anyhow::Result<Vec<Hardware>> {
        self.instance.get_hardware()
    }
}
