namespace lhwm_bridge;

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using LibreHardwareMonitor.Hardware;
using static lhwm_bridge.FFI;


public static class FFI
{

    /// <summary>
    /// Free a string
    /// </summary>
    /// <param name="ptr">The string to free</param>
    [UnmanagedCallersOnly(EntryPoint = "free_string", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void FreeString(Utf8Ptr ptr)
    {
        ptr.Dispose();
    }

    /// <summary>
    /// Free a shared array
    /// </summary>
    /// <param name="ptr">The array to free</param>
    [UnmanagedCallersOnly(EntryPoint = "free_shared_array", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void FreeSharedArray(SharedFfiArrayPtr ptr)
    {
        ptr.Dispose();
    }

    /// <summary>
    /// Create a new computer instance
    /// </summary>
    /// <returns>Created computer or an error</returns>
    [UnmanagedCallersOnly(EntryPoint = "create_computer", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static ComputerResult CreateComputer()
    {
        try
        {
            Computer computer = new Computer { };
            computer.Open();
            return ComputerResult.Success(new ComputerPtr(computer));
        }
        catch (Exception ex)
        {
            string message = ex.ToString();
            Utf8Ptr ptr = new Utf8Ptr(message);
            return ComputerResult.Error(ptr);
        }
    }

    /// <summary>
    /// Update a computer instance
    /// </summary>
    /// <param name="ptr">The computer instance to free</param>
    [UnmanagedCallersOnly(EntryPoint = "update_computer", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void UpdateComputer(ComputerPtr ptr)
    {
        ptr.Update();
    }

    /// <summary>
    /// Free a computer instance
    /// </summary>
    /// <param name="ptr">The computer instance to free</param>
    [UnmanagedCallersOnly(EntryPoint = "free_computer", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void FreeComputer(ComputerPtr ptr)
    {
        ptr.Dispose();
    }

    /// <summary>
    /// Update a computer instance options
    /// </summary>
    /// <param name="ptr">The computer instance to set the options for</param>
    [UnmanagedCallersOnly(EntryPoint = "set_computer_options", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void SetComputerOptions(ComputerPtr ptr, ComputerOptions options)
    {
        ptr.SetOptions(options);
    }


    [UnmanagedCallersOnly(EntryPoint = "get_computer_hardware", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static SharedFfiArray<HardwarePtr> getComputerHardware(ComputerPtr ptr)
    {
        return ptr.getHardware();
    }

    [UnmanagedCallersOnly(EntryPoint = "get_hardware_identifier", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static Utf8Ptr GetHardwareIdentifier(HardwarePtr ptr)
    {
        return ptr.getIdentifier();
    }

    [UnmanagedCallersOnly(EntryPoint = "get_hardware_name", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static Utf8Ptr GetHardwareName(HardwarePtr ptr)
    {
        return ptr.getName();
    }

    [UnmanagedCallersOnly(EntryPoint = "get_hardware_type", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static int GetHardwareType(HardwarePtr ptr)
    {
        return ptr.getType();
    }

    [UnmanagedCallersOnly(EntryPoint = "get_hardware_children", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static SharedFfiArray<HardwarePtr> GetHardwareChildren(HardwarePtr ptr)
    {
        return ptr.getSubHardware();
    }



    [UnmanagedCallersOnly(EntryPoint = "get_hardware_sensors", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static SharedFfiArray<SensorPtr> GetHardwareSensors(HardwarePtr ptr)
    {
        return ptr.getSensors();
    }


    [UnmanagedCallersOnly(EntryPoint = "update_hardware", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void UpdateHardware(HardwarePtr ptr)
    {
        ptr.Update();
    }

    [UnmanagedCallersOnly(EntryPoint = "free_hardware", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void FreeHardware(HardwarePtr ptr)
    {
        ptr.Dispose();
    }


    [UnmanagedCallersOnly(EntryPoint = "get_sensor_hardware", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static HardwarePtr GetSensorHardware(SensorPtr ptr)
    {
        return ptr.Hardware();
    }


    [UnmanagedCallersOnly(EntryPoint = "get_sensor_identifier", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static Utf8Ptr GetSensorIdentifier(SensorPtr ptr)
    {
        return ptr.getIdentifier();
    }


    [UnmanagedCallersOnly(EntryPoint = "get_sensor_name", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static Utf8Ptr GetSensorName(SensorPtr ptr)
    {
        return ptr.getName();
    }

    [UnmanagedCallersOnly(EntryPoint = "get_sensor_type", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static int GetSensorType(SensorPtr ptr)
    {
        return ptr.getType();
    }


    [UnmanagedCallersOnly(EntryPoint = "get_sensor_value", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static float GetSensorValue(SensorPtr ptr)
    {
        return ptr.getValue() ?? float.NaN;
    }


    [UnmanagedCallersOnly(EntryPoint = "get_sensor_min", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static float getSensorMin(SensorPtr ptr)
    {
        return ptr.getMin() ?? float.NaN;
    }


    [UnmanagedCallersOnly(EntryPoint = "get_sensor_max", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static float getSensorMax(SensorPtr ptr)
    {
        return ptr.getMax() ?? float.NaN;
    }


    [UnmanagedCallersOnly(EntryPoint = "update_sensor", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void UpdateSensor(SensorPtr ptr)
    {
        ptr.Update();
    }

    [UnmanagedCallersOnly(EntryPoint = "free_sensor", CallConvs = new[] { typeof(CallConvCdecl) })]
    public static void FreeSensor(SensorPtr ptr)
    {
        ptr.Dispose();
    }


}