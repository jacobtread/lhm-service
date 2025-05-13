namespace lhwm_bridge;

using System;
using System.Linq;
using System.Runtime.InteropServices;
using LibreHardwareMonitor.Hardware;

[StructLayout(LayoutKind.Sequential)]
public struct ComputerOptions
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


[StructLayout(LayoutKind.Sequential)]
public readonly struct ComputerPtr : IDisposable
{
    /// <summary>
    /// Pointer to the GCHandle for the computer instance
    /// </summary>
    private readonly IntPtr data;

    /// <summary>
    /// Create a new computer pointer from a computer item
    /// </summary>
    /// <param name="computer">The computer item</param>
    public ComputerPtr(Computer computer)
    {
        data = GCHandle.ToIntPtr(GCHandle.Alloc(computer));
    }

    /// <summary>
    /// Get the references computer item
    /// </summary>
    Computer computer
    {
        get
        {
            var handle = GCHandle.FromIntPtr(data);

            if (handle.Target is Computer computer)
            {
                return computer;
            }

            throw new InvalidCastException("GCHandle target is not of type Icomputer.");
        }
    }

    /// <summary>
    /// Update all the options for the computer instance
    /// </summary>
    /// <param name="options"></param>
    public void SetOptions(ComputerOptions options)
    {
        Computer computer = this.computer;
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

    /// <summary>
    /// Get all hardware children
    /// </summary>
    /// <returns></returns>
    public SharedFfiArray<HardwarePtr> getHardware()
    {
        HardwarePtr[] hardwarePtrs = Array.ConvertAll(
            computer.Hardware.ToArray(),
            subhardware => new HardwarePtr(subhardware)
        );

        return new SharedFfiArray<HardwarePtr>(hardwarePtrs);
    }

    /// <summary>
    /// Refresh the computer and all hardware
    /// </summary>
    public void Update()
    {
        computer.Accept(new UpdateVisitor());
    }

    /// <summary>
    /// Dispose of the pointer for this computer item and closes it
    /// </summary>
    public void Dispose()
    {
        var handle = GCHandle.FromIntPtr(data);

        if (handle.Target is Computer computer)
        {
            computer.Close();
        }

        handle.Free();
    }
}