namespace lhwm_bridge;

using System;
using System.Runtime.InteropServices;
using LibreHardwareMonitor.Hardware;
using System.Runtime.CompilerServices;
using System.Diagnostics;
using System.Reflection.Metadata;
using static lhwm_bridge.Exported;
using System.Collections.Generic;

public static class Exported
{

    public class UpdateVisitor : IVisitor
    {
        public void VisitComputer(IComputer computer)
        {
            computer.Traverse(this);
        }
        public void VisitHardware(IHardware hardware)
        {
            hardware.Update();
            foreach (IHardware subHardware in hardware.SubHardware) subHardware.Accept(this);
        }
        public void VisitSensor(ISensor sensor) { }
        public void VisitParameter(IParameter parameter) { }
    }

    /// <summary>
    /// Create and initialize a new instance of the computer for 
    /// obtaining sensor information
    /// </summary>
    /// <returns>Pointer to access the create computer instance</returns>
    [UnmanagedCallersOnly(EntryPoint = "create_computer_instance", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static IntPtr CreateComputerInstance(RComputerOptions options)
    {
        try
        {
            Computer computer = new Computer
            {
                IsBatteryEnabled = options.battery_enabled,
                IsControllerEnabled = options.controller_enabled,
                IsCpuEnabled = options.cpu_enabled,
                IsGpuEnabled = options.gpu_enabled,
                IsMemoryEnabled = options.memory_enabled,
                IsMotherboardEnabled = options.motherboard_enabled,
                IsNetworkEnabled = options.network_enabled,
                IsPsuEnabled = options.psu_enabled,
                IsStorageEnabled = options.storage_enabled, 
            };

            computer.Open();
            return GCHandle.ToIntPtr(GCHandle.Alloc(computer));
        }
        catch
        {
            return IntPtr.Zero;
        }
    }

    /// <summary>
    /// Closes and frees computer instance using its pointer
    /// </summary>
    /// <returns></returns>
    [UnmanagedCallersOnly(EntryPoint = "free_computer_instance", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void FreeComputerInstance(IntPtr instance)
    {
        try
        {
            if (instance == IntPtr.Zero) return;

            var handle = GCHandle.FromIntPtr(instance);
            if (handle.Target is Computer computer)
            {
                computer.Close();
            }
            handle.Free();
        }
        catch
        {
        }
    }


    /// <summary>
    /// Updates the list of sensor instances for the computer instance
    /// </summary>
    /// <returns></returns>
    [UnmanagedCallersOnly(EntryPoint = "update_computer_instance_options", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void UpdateComputerInstanceOptions(IntPtr instance, RComputerOptions options)
    {
        try
        {
            // Ignore nullptr
            if (instance == IntPtr.Zero) return;

            var handle = GCHandle.FromIntPtr(instance);
            if (handle.Target is Computer computer)
            {
                computer.IsBatteryEnabled = options.battery_enabled;
                computer.IsControllerEnabled = options.controller_enabled;
                computer.IsCpuEnabled = options.cpu_enabled;
                computer.IsGpuEnabled = options.gpu_enabled;
                computer.IsMemoryEnabled = options.memory_enabled;
                computer.IsMotherboardEnabled = options.motherboard_enabled;
                computer.IsNetworkEnabled = options.network_enabled;
                computer.IsPsuEnabled = options.psu_enabled;
                computer.IsStorageEnabled = options.storage_enabled;
            }
        }
        catch
        {
        }
    }

    /// <summary>
    /// Updates the list of sensor instances for the computer instance
    /// </summary>
    /// <returns></returns>
    [UnmanagedCallersOnly(EntryPoint = "update_computer_instance", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void UpdateComputerInstance(IntPtr instance)
    {
        try
        {
            // Ignore nullptr
            if (instance == IntPtr.Zero) return;

            var handle = GCHandle.FromIntPtr(instance);
            if (handle.Target is Computer computer)
            {
                computer.Accept(new UpdateVisitor());
            }
        }
        catch
        {
        }
    }

    /// <summary>
    /// Rust compatible options set
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct RComputerOptions
    {
        public bool battery_enabled;
        public bool controller_enabled;
        public bool cpu_enabled;
        public bool gpu_enabled;
        public bool memory_enabled;
        public bool motherboard_enabled;
        public bool network_enabled;
        public bool psu_enabled;
        public bool storage_enabled;
    }

    /// <summary>
    /// Rust compatible array
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct RArray
    {
        /// <summary>
        /// Length of the array
        /// </summary>
        public int length;

        /// <summary>
        /// Pointer to the data for the array
        /// </summary>
        public IntPtr data;
    }

    /// <summary>
    /// Hardware item
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct Hardware
    {
        // UTF-8 string for the hardware item name 
        public IntPtr name;
        // Type of hardware
        public int type;
        // Nested hardware Hardware[]
        public RArray children;
        // Sensors on the hardware itself Sensor[]
        public RArray sensors;
    }

    /// <summary>
    /// Sensor item
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct Sensor
    {
        public IntPtr name;
        public int type;
        public float value;
    }

    /// <summary>
    /// Create a hardware item from the hardware interface
    /// </summary>
    /// <param name="hardware">The hardware interface</param>
    /// <returns></returns>
    private static Hardware CreateHardwareItem(IHardware hardware, int level)
    {
        var children = new List<Hardware>();
        var sensors = new List<Sensor>();

        // Collect children hardware
        foreach (IHardware subhardware in hardware.SubHardware)
        {
            children.Add(CreateHardwareItem(subhardware, level + 1));
        }

        foreach (ISensor sensor in hardware.Sensors)
        {
            sensors.Add(new Sensor
            {
                name = StringToUtf8(sensor.Name),
                type = (int)sensor.SensorType,
                value = sensor.Value ?? float.NaN
            });
        }

        var children_array = RArrayFromList(children);
        var sensors_array = RArrayFromList(sensors);

        return new Hardware
        {
            name = StringToUtf8(hardware.Name),
            type = (int)hardware.HardwareType,
            children = children_array,
            sensors = sensors_array
        };
    }

    private static void FreeHardwareItem(Hardware hardware)
    {
        // Free the hardware name string
        Marshal.FreeHGlobal(hardware.name);
        FreeSensorArray(hardware.sensors);
        FreeHardwareArray(hardware.children);
    }


    private static void FreeSensorArray(RArray sensors)
    {

        // Iterate sensors to free each sensor value
        int sizeOfSensor = Marshal.SizeOf<Sensor>();
        for (int i = 0; i < sensors.length; i++)
        {
            // Get the sensor item at the current index
            IntPtr itemPtr = sensors.data + i * sizeOfSensor;
            Sensor sensor = Marshal.PtrToStructure<Sensor>(itemPtr);

            // Free the sensor name
            Marshal.FreeHGlobal(sensor.name);
        }

        // Free the array data
        Marshal.FreeHGlobal(sensors.data);

    }

    private static void FreeHardwareArray(RArray items)
    {

        // Iterate hardware to free each hardware value
        int sizeOfHardware = Marshal.SizeOf<Hardware>();
        for (int i = 0; i < items.length; i++)
        {
            // Get the child item at the current index
            IntPtr itemPtr = items.data + i * sizeOfHardware;
            Hardware child = Marshal.PtrToStructure<Hardware>(itemPtr);

            // Free the hardware child item
            FreeHardwareItem(child);
        }

        // Free the array data
        Marshal.FreeHGlobal(items.data);
    }

    /// <summary>
    /// Get the list of hardware 
    /// </summary>
    /// <returns></returns>
    [UnmanagedCallersOnly(EntryPoint = "get_computer_hardware", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static RArray GetComputerHardware(IntPtr instance)
    {
        var hardwareList = new List<Hardware>();

        try
        {
            // Invalid null instance
            if (instance == IntPtr.Zero)
            {
                return RArrayFromList(hardwareList);
            }

            var handle = GCHandle.FromIntPtr(instance);

            // Ensure the target is actually the computer instance
            if (handle.Target is not Computer computer)
            {
                return RArrayFromList(hardwareList);
            }

            // Iterate all the hardware
            foreach (IHardware hardware in computer.Hardware)
            {
                hardwareList.Add(CreateHardwareItem(hardware, 0));
            }

            return RArrayFromList(hardwareList);
        }
        catch
        {
        }

        return RArrayFromList(hardwareList);
    }

    /// <summary>
    /// Free an array of hardware items
    /// </summary>
    /// <param name="hardwareArray">Hardware to free</param>
    [UnmanagedCallersOnly(EntryPoint = "free_hardware_array", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void FreeHardwareArrayFFI(RArray hardware_array)
    {
        FreeHardwareArray(hardware_array);
    }

    /// <summary>
    /// Convert a C# string into a null terminated string
    /// </summary>
    /// <param name="str">The input C# string</param>
    /// <returns>Pointer to </returns>
    private static IntPtr StringToUtf8(string str)
    {
        if (string.IsNullOrEmpty(str))
            return IntPtr.Zero;

        byte[] utf8Bytes = System.Text.Encoding.UTF8.GetBytes(str + '\0');
        IntPtr ptr = Marshal.AllocHGlobal(utf8Bytes.Length);
        Marshal.Copy(utf8Bytes, 0, ptr, utf8Bytes.Length);
        return ptr;
    }


    public static RArray RArrayFromList<T>(List<T> list) where T : struct
    {
        int sizeOfT = Marshal.SizeOf<T>();
        int count = list.Count;
        IntPtr dataPtr = Marshal.AllocHGlobal(sizeOfT * count);

        // Copy data into unmanaged memory
        for (int i = 0; i < count; i++)
        {
            Marshal.StructureToPtr(list[i], dataPtr + i * sizeOfT, false);
        }

        return new RArray
        {
            data = dataPtr,
            length = count
        };
    }
}
