namespace lhwm_bridge;

// Define layout as explicit for union-like behavior
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Explicit)]
public struct ComputerResultUnion
{
    [FieldOffset(0)]
    public ComputerPtr OkValue;

    [FieldOffset(0)]
    public Utf8Ptr ErrValue;
}

[StructLayout(LayoutKind.Sequential)]
public struct ComputerResult
{
    public bool IsOk; // 1 for Ok, 0 for Err
    public ComputerResultUnion Data;

    public static ComputerResult Success(ComputerPtr value) => new() { IsOk = true, Data = new ComputerResultUnion { OkValue = value } };
    public static ComputerResult Error(Utf8Ptr err) => new() { IsOk = false, Data = new ComputerResultUnion { ErrValue = err } };
}