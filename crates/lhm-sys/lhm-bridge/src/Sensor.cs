namespace lhwm_bridge;

using System.Runtime.InteropServices;
using LibreHardwareMonitor.Hardware;

[StructLayout(LayoutKind.Sequential)]
public readonly struct SensorPtr : IDisposable
{
    /// <summary>
    /// Pointer to the GCHandle for the hardware instance
    /// </summary>
    private readonly IntPtr data;

    /// <summary>
    /// Create a new sensor pointer from a sensor item
    /// </summary>
    /// <param name="sensor">The sensor item</param>
    public SensorPtr(ISensor sensor)
    {
        data = GCHandle.ToIntPtr(GCHandle.Alloc(sensor));
    }

    /// <summary>
    /// Get the references sensor item
    /// </summary>
    ISensor sensor
    {
        get
        {
            var handle = GCHandle.FromIntPtr(data);

            if (handle.Target is ISensor sensor)
            {
                return sensor;
            }

            throw new InvalidCastException("GCHandle target is not of type ISensor.");
        }
    }


    /// <summary>
    /// Get the parent hardware item
    /// </summary>
    /// <returns>THe hardware item</returns>
    public HardwarePtr Hardware()
    {
        return new HardwarePtr(sensor.Hardware);
    }

    /// <summary>
    /// Get the identifier of a hardware item
    /// </summary>
    /// <returns>The identifier</returns>
    public Utf8Ptr getIdentifier()
    {
        return new Utf8Ptr(sensor.Identifier.ToString());
    }

    /// <summary>
    /// Get the name of the hardware item
    /// </summary>
    /// <returns>The name</returns>
    public Utf8Ptr getName()
    {
        return new Utf8Ptr(sensor.Name);
    }


    /// <summary>
    /// Get the name of the hardware item
    /// </summary>
    /// <returns>The current sensor value</returns>
    public float? getValue()
    {
        return sensor.Value;
    }


    /// <summary>
    /// Gets a minimum value recorded for the given sensor.
    /// </summary>
    /// <returns>The minimum recorded value</returns>
    public float? getMin()
    {
        return sensor.Min;
    }


    /// <summary>
    /// Gets a minimum value recorded for the given sensor.
    /// </summary>
    /// <returns>The minimum recorded value</returns>
    public float? getMax()
    {
        return sensor.Max;
    }

    /// <summary>
    /// Update the parent hardware to refresh the 
    /// sensor value
    /// </summary>
    public void Update()
    {
        sensor.Hardware.Update();
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