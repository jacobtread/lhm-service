use dlopen::wrapper::{Container, WrapperApi};
use lhm_shared::{Hardware, HardwareType, Sensor, SensorType};
use std::{
    ffi::{CStr, c_char, c_void},
    marker::PhantomData,
    path::Path,
    ptr::null,
    sync::Arc,
};

use lhm_shared::ComputerOptions;

#[repr(C)]
pub(crate) struct RComputerOptions {
    battery_enabled: bool,
    controller_enabled: bool,
    cpu_enabled: bool,
    gpu_enabled: bool,
    memory_enabled: bool,
    motherboard_enabled: bool,
    network_enabled: bool,
    psu_enabled: bool,
    storage_enabled: bool,
}

impl From<ComputerOptions> for RComputerOptions {
    fn from(value: ComputerOptions) -> Self {
        Self {
            battery_enabled: value.battery_enabled,
            controller_enabled: value.controller_enabled,
            cpu_enabled: value.cpu_enabled,
            gpu_enabled: value.gpu_enabled,
            memory_enabled: value.memory_enabled,
            motherboard_enabled: value.motherboard_enabled,
            network_enabled: value.network_enabled,
            psu_enabled: value.psu_enabled,
            storage_enabled: value.storage_enabled,
        }
    }
}

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
pub(crate) struct BridgeApi {
    create_computer_instance: unsafe extern "C" fn(options: RComputerOptions) -> *const c_void,
    free_computer_instance: unsafe extern "C" fn(instance: *const c_void),
    update_computer_instance: unsafe extern "C" fn(instance: *const c_void),
    update_computer_instance_options:
        unsafe extern "C" fn(instance: *const c_void, options: RComputerOptions),
    get_computer_hardware: unsafe extern "C" fn(instance: *const c_void) -> RArray<CHardware>,
    free_hardware_array: unsafe extern "C" fn(hardware_array: RArray<CHardware>),
}

const EMBEDDED_DLL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lhm-bridge.dll"));
const DLL_NAME: &str = "lhm-bridge.dll";

/// Initialize the lhm-bridge.dll ensuring that the file exists
fn init_bridge_dll() {
    let dll_path = Path::new(DLL_NAME);
    if !dll_path.exists() {
        std::fs::write(dll_path, EMBEDDED_DLL).expect("failed to write embedded dll");
    }
}

/// Loads the bridge DLL
pub(crate) fn load_bridge_dll() -> Result<BridgeContainer, dlopen::Error> {
    // Ensure DLL exists
    init_bridge_dll();

    // Load the DLL container
    unsafe { Container::load(DLL_NAME) }
}

pub(crate) type BridgeContainer = Container<BridgeApi>;
pub(crate) type SharedBridgeContainer = Arc<BridgeContainer>;

pub struct ComputerInstance {
    bridge: SharedBridgeContainer,
    instance: *const c_void,
}

impl ComputerInstance {
    pub fn create(bridge: SharedBridgeContainer, options: RComputerOptions) -> Self {
        let instance = unsafe { bridge.create_computer_instance(options) };

        if instance.is_null() {
            panic!("failed to create instance")
        }

        Self { bridge, instance }
    }

    pub fn update(&mut self) {
        unsafe {
            self.bridge.update_computer_instance(self.instance);
        }
    }

    pub fn update_options(&mut self, options: RComputerOptions) {
        unsafe {
            self.bridge
                .update_computer_instance_options(self.instance, options);
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

impl Drop for ComputerInstance {
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

        let ty = HardwareType::from(item.ty);
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
        let ty = SensorType::from(item.ty);

        sensors.push(Sensor {
            name,
            ty,
            value: item.value,
        });
    }

    sensors
}
