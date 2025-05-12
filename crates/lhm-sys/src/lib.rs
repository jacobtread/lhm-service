#[macro_use]
extern crate dlopen_derive;

mod ffi;

use std::rc::Rc;

use ffi::{ComputerInstance, SharedBridgeContainer, load_bridge_dll};
use lhm_shared::Hardware;

/// Instance of the bridge library used for creating computer instances
#[derive(Clone)]
pub struct Bridge {
    inner: SharedBridgeContainer,
}

impl Bridge {
    pub fn load() -> Result<Bridge, dlopen::Error> {
        let container = load_bridge_dll()?;
        Ok(Self {
            inner: Rc::new(container),
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
    pub fn create(bridge: &Bridge) -> Self {
        let instance = ComputerInstance::create(bridge.inner.clone());
        Self { instance }
    }

    pub fn update(&mut self) {
        self.instance.update();
    }

    pub fn get_hardware(&mut self) -> anyhow::Result<Vec<Hardware>> {
        self.instance.get_hardware()
    }
}
