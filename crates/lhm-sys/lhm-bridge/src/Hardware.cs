namespace lhwm_bridge;

using System;
using System.Runtime.InteropServices;
using LibreHardwareMonitor.Hardware;

[StructLayout(LayoutKind.Sequential)]
public readonly struct HardwarePtr : IDisposable
{
    /// <summary>
    /// Pointer to the GCHandle for the hardware instance
    /// </summary>
    private readonly IntPtr data;

    /// <summary>
    /// Create a new hardware pointer from a hardware item
    /// </summary>
    /// <param name="hardware">The hardware item</param>
    public HardwarePtr(IHardware hardware)
    {
        data = GCHandle.ToIntPtr(GCHandle.Alloc(hardware));
    }

    /// <summary>
    /// Get the references hardware item
    /// </summary>
    IHardware hardware
    {
        get
        {
            var handle = GCHandle.FromIntPtr(data);

            if (handle.Target is IHardware hardware)
            {
                return hardware;
            }

            throw new InvalidCastException("GCHandle target is not of type IHardware.");
        }
    }

    /// <summary>
    /// Get all hardware children
    /// </summary>
    /// <returns></returns>
    public SharedFfiArray<HardwarePtr> getSubHardware()
    {
        HardwarePtr[] hardwarePtrs = Array.ConvertAll(
            hardware.SubHardware,
            subhardware => new HardwarePtr(subhardware)
        );

        return new SharedFfiArray<HardwarePtr>(hardwarePtrs);
    }

    /// <summary>
    /// Get all the hardware sensors
    /// </summary>
    /// <returns></returns>
    public SharedFfiArray<SensorPtr> getSensors()
    {
        SensorPtr[] hardwarePtrs = Array.ConvertAll(
            hardware.Sensors,
            sensor => new SensorPtr(sensor)
        );

        return new SharedFfiArray<SensorPtr>(hardwarePtrs);
    }


    /// <summary>
    /// Get the identifier of a hardware item
    /// </summary>
    /// <returns>The identifier</returns>
    public Utf8Ptr getIdentifier()
    {
        return new Utf8Ptr(hardware.Identifier.ToString());
    }

    /// <summary>
    /// Get the name of the hardware item
    /// </summary>
    /// <returns>The name</returns>
    public Utf8Ptr getName()
    {
        return new Utf8Ptr(hardware.Name);
    }

    public int getType()
    {
        return (int)hardware.HardwareType;
    }


    /// <summary>
    /// Refresh the information in the sensors 
    /// for this hardware item
    /// </summary>
    public void Update()
    {
        hardware.Update();
    }

    /// <summary>
    /// Dispose of the pointer for this hardware item
    /// </summary>
    public void Dispose()
    {
        var handle = GCHandle.FromIntPtr(data);
        handle.Free();
    }
}