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

use dlopen::raw::Library;
use std::sync::Arc;
use std::{
    ffi::{CStr, OsStr, c_void},
    marker::PhantomData,
    path::Path,
};

#[repr(C)]
pub struct ComputerOptions {
    pub battery_enabled: bool,
    pub controller_enabled: bool,
    pub cpu_enabled: bool,
    pub gpu_enabled: bool,
    pub memory_enabled: bool,
    pub motherboard_enabled: bool,
    pub network_enabled: bool,
    pub psu_enabled: bool,
    pub storage_enabled: bool,
}

type HardwarePtr = *const c_void;
type SensorPtr = *const c_void;
type ComputerPtr = *const c_void;
type Utf8Ptr = *const c_void;

#[repr(C)]
struct SharedFfiArrayPtr {
    data: *const c_void,
    handle: *const c_void,
}

#[repr(C)]
struct SharedFfiArray<T> {
    length: u32,
    ptr: SharedFfiArrayPtr,
    _phantom: PhantomData<T>,
}

#[repr(C)]
union FfiResultUnion<O: Copy, E: Copy> {
    ok_value: O,
    err_value: E,
}

#[repr(C)]
struct FfiResult<O: Copy, E: Copy> {
    is_ok: bool,
    data: FfiResultUnion<O, E>,
}

type FreeString = unsafe extern "C" fn(ptr: Utf8Ptr);
type FreeSharedArray = unsafe extern "C" fn(ptr: SharedFfiArrayPtr);
type CreateComputer = unsafe extern "C" fn() -> FfiResult<ComputerPtr, Utf8Ptr>;
type UpdateComputer = unsafe extern "C" fn(ptr: ComputerPtr);
type FreeComputer = unsafe extern "C" fn(ptr: ComputerPtr);
type SetComputerOptions = unsafe extern "C" fn(ptr: ComputerPtr, options: ComputerOptions);
type GetComputerHardware = unsafe extern "C" fn(ptr: ComputerPtr) -> SharedFfiArray<HardwarePtr>;
type GetHardwareIdentifier = unsafe extern "C" fn(ptr: HardwarePtr) -> Utf8Ptr;
type GetHardwareName = unsafe extern "C" fn(ptr: HardwarePtr) -> Utf8Ptr;
type GetHardwareType = unsafe extern "C" fn(ptr: HardwarePtr) -> u32;
type GetHardwareChildren = unsafe extern "C" fn(ptr: HardwarePtr) -> SharedFfiArray<HardwarePtr>;
type GetHardwareSensors = unsafe extern "C" fn(ptr: HardwarePtr) -> SharedFfiArray<SensorPtr>;
type UpdateHardware = unsafe extern "C" fn(ptr: HardwarePtr);
type FreeHardware = unsafe extern "C" fn(ptr: HardwarePtr);
type GetSensorHardware = unsafe extern "C" fn(ptr: SensorPtr) -> HardwarePtr;
type GetSensorIdentifier = unsafe extern "C" fn(ptr: SensorPtr) -> Utf8Ptr;
type GetSensorName = unsafe extern "C" fn(ptr: SensorPtr) -> Utf8Ptr;
type GetSensorType = unsafe extern "C" fn(ptr: SensorPtr) -> u32;
type GetSensorValue = unsafe extern "C" fn(ptr: SensorPtr) -> f32;
type GetSensorMin = unsafe extern "C" fn(ptr: SensorPtr) -> f32;
type GetSensorMax = unsafe extern "C" fn(ptr: SensorPtr) -> f32;
type UpdateSensor = unsafe extern "C" fn(ptr: SensorPtr);
type FreeSensor = unsafe extern "C" fn(ptr: SensorPtr);

pub type SharedApi = Arc<Api>;

pub struct Api {
    #[allow(unused)]
    library: Library,
    free_string: FreeString,
    free_shared_array: FreeSharedArray,
    create_computer: CreateComputer,
    update_computer: UpdateComputer,
    free_computer: FreeComputer,
    set_computer_options: SetComputerOptions,
    get_computer_hardware: GetComputerHardware,
    get_hardware_identifier: GetHardwareIdentifier,
    get_hardware_name: GetHardwareName,
    get_hardware_type: GetHardwareType,
    get_hardware_children: GetHardwareChildren,
    get_hardware_sensors: GetHardwareSensors,
    update_hardware: UpdateHardware,
    free_hardware: FreeHardware,
    get_sensor_hardware: GetSensorHardware,
    get_sensor_identifier: GetSensorIdentifier,
    get_sensor_name: GetSensorName,
    get_sensor_type: GetSensorType,
    get_sensor_value: GetSensorValue,
    get_sensor_min: GetSensorMin,
    get_sensor_max: GetSensorMax,
    update_sensor: UpdateSensor,
    free_sensor: FreeSensor,
}

impl Api {
    pub fn load() -> Result<SharedApi, dlopen::Error> {
        // Ensure DLL exists
        init_bridge_dll();

        let library = Library::open(DLL_NAME)?;

        let free_string = unsafe { library.symbol("free_string") }?;
        let free_shared_array = unsafe { library.symbol("free_shared_array") }?;
        let create_computer = unsafe { library.symbol("create_computer") }?;
        let update_computer = unsafe { library.symbol("update_computer") }?;
        let free_computer = unsafe { library.symbol("free_computer") }?;
        let set_computer_options = unsafe { library.symbol("set_computer_options") }?;
        let get_computer_hardware = unsafe { library.symbol("get_computer_hardware") }?;
        let get_hardware_identifier = unsafe { library.symbol("get_hardware_identifier") }?;
        let get_hardware_type = unsafe { library.symbol("get_hardware_type") }?;
        let get_hardware_name = unsafe { library.symbol("get_hardware_name") }?;
        let get_hardware_children = unsafe { library.symbol("get_hardware_children") }?;
        let get_hardware_sensors = unsafe { library.symbol("get_hardware_sensors") }?;
        let update_hardware = unsafe { library.symbol("update_hardware") }?;
        let free_hardware = unsafe { library.symbol("free_hardware") }?;
        let get_sensor_hardware = unsafe { library.symbol("get_sensor_hardware") }?;
        let get_sensor_identifier = unsafe { library.symbol("get_sensor_identifier") }?;
        let get_sensor_name = unsafe { library.symbol("get_sensor_name") }?;
        let get_sensor_type = unsafe { library.symbol("get_sensor_type") }?;
        let get_sensor_value = unsafe { library.symbol("get_sensor_value") }?;
        let get_sensor_min = unsafe { library.symbol("get_sensor_min") }?;
        let get_sensor_max = unsafe { library.symbol("get_sensor_max") }?;
        let update_sensor = unsafe { library.symbol("update_sensor") }?;
        let free_sensor = unsafe { library.symbol("free_sensor") }?;

        Ok(Arc::new(Api {
            library,
            free_string,
            free_shared_array,
            create_computer,
            update_computer,
            free_computer,
            set_computer_options,
            get_computer_hardware,
            get_hardware_identifier,
            get_hardware_type,
            get_hardware_name,
            get_hardware_children,
            get_hardware_sensors,
            update_hardware,
            free_hardware,
            get_sensor_hardware,
            get_sensor_identifier,
            get_sensor_type,
            get_sensor_name,
            get_sensor_value,
            get_sensor_min,
            get_sensor_max,
            update_sensor,
            free_sensor,
        }))
    }
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

pub struct Computer {
    api: SharedApi,
    ptr: ComputerPtr,
}

impl Computer {
    pub fn create(bridge: SharedApi) -> std::io::Result<Self> {
        let result = unsafe { (bridge.create_computer)() };

        if !result.is_ok {
            let err_ptr = unsafe { result.data.err_value };
            let err = copy_string(&bridge, err_ptr);

            return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
        }

        let ptr = unsafe { result.data.ok_value };

        Ok(Self { api: bridge, ptr })
    }

    pub fn update(&mut self) {
        unsafe {
            (self.api.update_computer)(self.ptr);
        }
    }

    pub fn set_options(&mut self, options: ComputerOptions) {
        unsafe {
            (self.api.set_computer_options)(self.ptr, options);
        }
    }

    pub fn hardware(&self) -> Vec<Hardware> {
        // Get the name
        let array = unsafe { (self.api.get_computer_hardware)(self.ptr) };

        // Get a a slice of the values
        let slice =
            unsafe { std::slice::from_raw_parts(array.ptr.data.cast(), array.length as usize) };

        // Create hardware items from the hardware pointers
        let children: Vec<Hardware> = slice
            .iter()
            .copied()
            .map(|hardware_ptr| Hardware {
                api: self.api.clone(),
                ptr: hardware_ptr,
            })
            .collect();

        // Free the provided array
        unsafe { (self.api.free_shared_array)(array.ptr) }

        children
    }
}

impl Drop for Computer {
    fn drop(&mut self) {
        unsafe {
            (self.api.free_computer)(self.ptr);
        }
    }
}

/// Copies a string then frees it
fn copy_string(api: &Api, value_ptr: Utf8Ptr) -> String {
    if value_ptr.is_null() {
        return String::new();
    }

    // Create a copy of the value
    let value = unsafe { CStr::from_ptr(value_ptr.cast()) };
    let value = value.to_string_lossy().to_string();

    // Free the provided string
    unsafe { (api.free_string)(value_ptr) }

    value
}

pub struct Hardware {
    api: SharedApi,
    ptr: HardwarePtr,
}

impl Hardware {
    pub fn identifier(&self) -> String {
        let value_ptr = unsafe { (self.api.get_hardware_identifier)(self.ptr) };
        copy_string(&self.api, value_ptr)
    }

    pub fn name(&self) -> String {
        let value_ptr = unsafe { (self.api.get_hardware_name)(self.ptr) };
        copy_string(&self.api, value_ptr)
    }

    pub fn get_type(&self) -> u32 {
        unsafe { (self.api.get_hardware_type)(self.ptr) }
    }

    pub fn get_children(&self) -> Vec<Hardware> {
        // Get the name
        let array = unsafe { (self.api.get_hardware_children)(self.ptr) };

        // Get a a slice of the values
        let slice =
            unsafe { std::slice::from_raw_parts(array.ptr.data.cast(), array.length as usize) };

        // Create hardware items from the hardware pointers
        let children: Vec<Hardware> = slice
            .iter()
            .map(|hardware_ptr| Hardware {
                api: self.api.clone(),
                ptr: *hardware_ptr,
            })
            .collect();

        // Free the provided array
        unsafe { (self.api.free_shared_array)(array.ptr) }

        children
    }

    pub fn sensors(&self) -> Vec<Sensor> {
        // Get the name
        let array = unsafe { (self.api.get_hardware_sensors)(self.ptr) };

        // Get a a slice of the values
        let slice =
            unsafe { std::slice::from_raw_parts(array.ptr.data.cast(), array.length as usize) };

        // Create hardware items from the hardware pointers
        let children: Vec<Sensor> = slice
            .iter()
            .map(|sensor_ptr| Sensor {
                api: self.api.clone(),
                ptr: *sensor_ptr,
            })
            .collect();

        // Free the provided array
        unsafe { (self.api.free_shared_array)(array.ptr) }

        children
    }

    pub fn update(&mut self) {
        unsafe { (self.api.update_hardware)(self.ptr) };
    }
}

impl Drop for Hardware {
    fn drop(&mut self) {
        unsafe { (self.api.free_hardware)(self.ptr) };
    }
}

pub struct Sensor {
    api: SharedApi,
    ptr: SensorPtr,
}

impl Sensor {
    pub fn hardware(&self) -> Hardware {
        let hardware_ptr = unsafe { (self.api.get_sensor_hardware)(self.ptr) };

        Hardware {
            api: self.api.clone(),
            ptr: hardware_ptr,
        }
    }

    pub fn identifier(&self) -> String {
        // Get the name
        let value_ptr = unsafe { (self.api.get_sensor_identifier)(self.ptr) };
        copy_string(&self.api, value_ptr)
    }

    pub fn name(&self) -> String {
        // Get the name
        let value_ptr = unsafe { (self.api.get_sensor_name)(self.ptr) };
        copy_string(&self.api, value_ptr)
    }

    pub fn get_type(&self) -> u32 {
        unsafe { (self.api.get_sensor_type)(self.ptr) }
    }

    pub fn value(&self) -> f32 {
        unsafe { (self.api.get_sensor_value)(self.ptr) }
    }

    pub fn min(&self) -> f32 {
        unsafe { (self.api.get_sensor_min)(self.ptr) }
    }

    pub fn max(&self) -> f32 {
        unsafe { (self.api.get_sensor_max)(self.ptr) }
    }

    pub fn update(&mut self) {
        unsafe { (self.api.update_sensor)(self.ptr) };
    }
}

impl Drop for Sensor {
    fn drop(&mut self) {
        unsafe { (self.api.free_sensor)(self.ptr) };
    }
}
