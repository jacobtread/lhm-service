use std::{
    ffi::{CStr, c_char, c_void},
    marker::PhantomData,
    ptr::null,
    rc::Rc,
};

use dlopen::wrapper::{Container, WrapperApi};
use num_enum::TryFromPrimitive;

use crate::{Hardware, HardwareType, Sensor, SensorType};

#[repr(C)]
struct RArray<T> {
    length: u32,
    data: *const c_void,
    _type: PhantomData<T>,
}

#[repr(C)]
struct CHardware {
    name: *const c_char,
    ty: u32,
    children: RArray<CHardware>,
    sensors: RArray<CSensor>,
}

#[repr(C)]
struct CSensor {
    name: *const c_char,
    ty: u32,
    value: f32,
}

#[derive(WrapperApi)]
struct BridgeApi {
    create_computer_instance: unsafe extern "C" fn() -> *const c_void,
    free_computer_instance: unsafe extern "C" fn(instance: *const c_void),
    update_computer_instance: unsafe extern "C" fn(instance: *const c_void),
    get_computer_hardware: unsafe extern "C" fn(instance: *const c_void) -> RArray<CHardware>,
    free_hardware_array: unsafe extern "C" fn(hardware_array: RArray<CHardware>),
}

#[derive(Clone)]
pub struct Bridge {
    inner: Rc<Container<BridgeApi>>,
}

impl Bridge {
    pub fn init() -> Self {
        let container: Container<BridgeApi> = unsafe { Container::load("lhm-bridge.dll") }
            .expect("Could not open library or load symbols");
        Self {
            inner: Rc::new(container),
        }
    }
}

pub struct Computer {
    bridge: Rc<Container<BridgeApi>>,
    instance: *const c_void,
}

impl Computer {
    pub fn create(bridge: &Bridge) -> Self {
        let instance = unsafe { bridge.inner.create_computer_instance() };

        if instance.is_null() {
            panic!("failed to create instance")
        }

        Self {
            bridge: bridge.inner.clone(),
            instance,
        }
    }

    pub fn update(&mut self) {
        unsafe {
            self.bridge.update_computer_instance(self.instance);
        }
    }

    pub fn get_hardware(&mut self) -> Vec<Hardware> {
        // Request the hardware array
        let hardware = unsafe { self.bridge.get_computer_hardware(self.instance) };

        // Create a safe rust copy
        let safe_hardware = copy_hardware_safe(&hardware);

        // Free the original array
        unsafe {
            self.bridge.free_hardware_array(hardware);
        }

        safe_hardware
    }
}

impl Drop for Computer {
    fn drop(&mut self) {
        unsafe {
            self.bridge.free_computer_instance(self.instance);
            self.instance = null();
        }
    }
}

/// Copy the C# hardware plain structure into the Rust [Hardware] struct
fn copy_hardware_safe(hardware: &RArray<CHardware>) -> Vec<Hardware> {
    let hardware_slice = unsafe {
        std::slice::from_raw_parts(hardware.data.cast::<CHardware>(), hardware.length as usize)
    };

    let mut hardware = Vec::new();

    for item in hardware_slice {
        let name = unsafe { CStr::from_ptr(item.name) };
        let name = name.to_string_lossy().into_owned();

        let ty = HardwareType::try_from_primitive(item.ty).unwrap();
        let children = copy_hardware_safe(&item.children);
        let sensors = copy_sensors_safe(&item.sensors);

        hardware.push(Hardware {
            name,
            ty,
            children,
            sensors,
        });
    }

    hardware
}

/// Copy the C# sensor plain structure into the Rust [Sensor] struct
fn copy_sensors_safe(sensors: &RArray<CSensor>) -> Vec<Sensor> {
    let sensors_slice = unsafe {
        std::slice::from_raw_parts(sensors.data.cast::<CSensor>(), sensors.length as usize)
    };

    let mut sensors = Vec::new();

    for item in sensors_slice {
        let name = unsafe { CStr::from_ptr(item.name) };
        let name = name.to_string_lossy().into_owned();
        let ty = SensorType::try_from_primitive(item.ty).unwrap();

        sensors.push(Sensor {
            name,
            ty,
            value: item.value,
        });
    }

    sensors
}
