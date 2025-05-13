
// Define layout as explicit for union-like behavior
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Explicit)]
public struct ResultUnion<O, E>
{
    [FieldOffset(0)]
    public O OkValue;

    [FieldOffset(0)]
    public E ErrValue;
}

[StructLayout(LayoutKind.Sequential)]
public struct Result<O, E>
{
    public bool IsOk; // 1 for Ok, 0 for Err
    public ResultUnion<O, E> Data;

    public static Result<O, E> Success(O value) => new() { IsOk = true, Data = new ResultUnion<O, E> { OkValue = value } };
    public static Result<O, E> Error(E err) => new() { IsOk = false, Data = new ResultUnion<O, E> { OkValue = err } };
}