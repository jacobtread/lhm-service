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

use std::{
    ffi::{CStr, c_void},
    marker::PhantomData,
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

/// This is required for C# to properly initialize, this symbol just needs to
/// make its way downstream into the final binary
#[cfg(feature = "static")]
#[used]
static FORCE_INCLUDE: unsafe extern "C" fn() = static_initialization;

#[link(name = "lhm-bridge")]
unsafe extern "C" {

    /// When statically linked we must ensure the NativeAOT_StaticInitialization symbol
    /// is present on the compiled binary
    #[cfg(feature = "static")]
    #[link_name = "NativeAOT_StaticInitialization"]
    pub fn static_initialization();

    unsafe fn free_string(ptr: Utf8Ptr);
    unsafe fn free_shared_array(ptr: SharedFfiArrayPtr);
    unsafe fn create_computer() -> FfiResult<ComputerPtr, Utf8Ptr>;
    unsafe fn update_computer(ptr: ComputerPtr);
    unsafe fn free_computer(ptr: ComputerPtr);
    unsafe fn set_computer_options(ptr: ComputerPtr, options: ComputerOptions);
    unsafe fn get_computer_hardware(ptr: ComputerPtr) -> SharedFfiArray<HardwarePtr>;
    unsafe fn get_hardware_identifier(ptr: HardwarePtr) -> Utf8Ptr;
    unsafe fn get_hardware_name(ptr: HardwarePtr) -> Utf8Ptr;
    unsafe fn get_hardware_type(ptr: HardwarePtr) -> i32;
    unsafe fn get_hardware_children(ptr: HardwarePtr) -> SharedFfiArray<HardwarePtr>;
    unsafe fn get_hardware_sensors(ptr: HardwarePtr) -> SharedFfiArray<SensorPtr>;
    unsafe fn update_hardware(ptr: HardwarePtr);
    unsafe fn free_hardware(ptr: HardwarePtr);
    unsafe fn get_sensor_hardware(ptr: SensorPtr) -> HardwarePtr;
    unsafe fn get_sensor_identifier(ptr: SensorPtr) -> Utf8Ptr;
    unsafe fn get_sensor_name(ptr: SensorPtr) -> Utf8Ptr;
    unsafe fn get_sensor_type(ptr: SensorPtr) -> i32;
    unsafe fn get_sensor_value(ptr: SensorPtr) -> f32;
    unsafe fn get_sensor_min(ptr: SensorPtr) -> f32;
    unsafe fn get_sensor_max(ptr: SensorPtr) -> f32;
    unsafe fn update_sensor(ptr: SensorPtr);
    unsafe fn free_sensor(ptr: SensorPtr);
}

/// Computer
///
/// This is your access to the hardware and all the sensors. You must call
/// [Self::update] at least once before [Self::hardware] will contain values
pub struct Computer {
    ptr: ComputerPtr,
}

impl Computer {
    /// Create a new instance of the
    pub fn create() -> std::io::Result<Self> {
        let result = unsafe { create_computer() };
        if !result.is_ok {
            let err_ptr = unsafe { result.data.err_value };
            let err = copy_string(err_ptr);

            return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
        }

        let ptr = unsafe { result.data.ok_value };

        Ok(Self { ptr })
    }

    /// Updates all the hardware and sensors
    pub fn update(&mut self) {
        unsafe {
            update_computer(self.ptr);
        }
    }

    /// Set the options for the computer
    pub fn set_options(&mut self, options: ComputerOptions) {
        unsafe {
            set_computer_options(self.ptr, options);
        }
    }

    /// Get a list of hardware from the computer
    pub fn hardware(&self) -> Vec<Hardware> {
        // Get the name
        let array = unsafe { get_computer_hardware(self.ptr) };

        // Get a a slice of the values
        let slice =
            unsafe { std::slice::from_raw_parts(array.ptr.data.cast(), array.length as usize) };

        // Create hardware items from the hardware pointers
        let children: Vec<Hardware> = slice
            .iter()
            .copied()
            .map(|hardware_ptr| Hardware { ptr: hardware_ptr })
            .collect();

        // Free the provided array
        unsafe { free_shared_array(array.ptr) }

        children
    }
}

impl Drop for Computer {
    fn drop(&mut self) {
        unsafe {
            free_computer(self.ptr);
        }
    }
}

/// Hardware item
pub struct Hardware {
    ptr: HardwarePtr,
}

impl Hardware {
    /// Get the identifier of the hardware item
    pub fn identifier(&self) -> String {
        copy_string(unsafe { get_hardware_identifier(self.ptr) })
    }

    /// Get the name of the hardware item
    pub fn name(&self) -> String {
        copy_string(unsafe { get_hardware_name(self.ptr) })
    }

    /// Get the type of hardware
    pub fn get_type(&self) -> i32 {
        unsafe { get_hardware_type(self.ptr) }
    }

    /// Get all children hardware for this item
    pub fn get_children(&self) -> Vec<Hardware> {
        // Get the name
        let array = unsafe { get_hardware_children(self.ptr) };

        // Get a a slice of the values
        let slice =
            unsafe { std::slice::from_raw_parts(array.ptr.data.cast(), array.length as usize) };

        // Create hardware items from the hardware pointers
        let children: Vec<Hardware> = slice
            .iter()
            .map(|hardware_ptr| Hardware { ptr: *hardware_ptr })
            .collect();

        // Free the provided array
        unsafe { free_shared_array(array.ptr) }

        children
    }

    /// Get all sensors that belong to this hardware item
    pub fn sensors(&self) -> Vec<Sensor> {
        // Get the name
        let array = unsafe { get_hardware_sensors(self.ptr) };

        // Get a a slice of the values
        let slice =
            unsafe { std::slice::from_raw_parts(array.ptr.data.cast(), array.length as usize) };

        // Create hardware items from the hardware pointers
        let children: Vec<Sensor> = slice
            .iter()
            .map(|sensor_ptr| Sensor { ptr: *sensor_ptr })
            .collect();

        // Free the provided array
        unsafe { free_shared_array(array.ptr) }

        children
    }

    /// Updates the hardware and all the sensors attached to it
    pub fn update(&mut self) {
        unsafe { update_hardware(self.ptr) };
    }
}

impl Drop for Hardware {
    fn drop(&mut self) {
        unsafe { free_hardware(self.ptr) };
    }
}

/// Sensor item
pub struct Sensor {
    ptr: SensorPtr,
}

impl Sensor {
    /// Get the parent hardware
    pub fn hardware(&self) -> Hardware {
        let ptr = unsafe { get_sensor_hardware(self.ptr) };
        Hardware { ptr }
    }

    //// Get the identifier of the sensor
    pub fn identifier(&self) -> String {
        copy_string(unsafe { get_sensor_identifier(self.ptr) })
    }

    /// Get the name of the sensor
    pub fn name(&self) -> String {
        copy_string(unsafe { get_sensor_name(self.ptr) })
    }

    /// Get the type of the sensor
    pub fn get_type(&self) -> i32 {
        unsafe { get_sensor_type(self.ptr) }
    }

    /// Get the last value for the sensor
    pub fn value(&self) -> f32 {
        unsafe { get_sensor_value(self.ptr) }
    }

    /// Get the minimum recorded value for the sensor
    pub fn min(&self) -> f32 {
        unsafe { get_sensor_min(self.ptr) }
    }

    /// Get the maximum recorded value for the sensor
    pub fn max(&self) -> f32 {
        unsafe { get_sensor_max(self.ptr) }
    }

    /// Updates the sensor value
    ///
    /// This updates the parent [Hardware] item to update the sensor
    /// as a sensor cannot be updated directly
    pub fn update(&mut self) {
        unsafe { update_sensor(self.ptr) };
    }
}

impl Drop for Sensor {
    fn drop(&mut self) {
        unsafe { free_sensor(self.ptr) };
    }
}

/// Copies a string then frees it
fn copy_string(value_ptr: Utf8Ptr) -> String {
    if value_ptr.is_null() {
        return String::new();
    }

    // Create a copy of the value
    let value = unsafe { CStr::from_ptr(value_ptr.cast()) };
    let value = value.to_string_lossy().to_string();

    // Free the provided string
    unsafe { free_string(value_ptr) }

    value
}
