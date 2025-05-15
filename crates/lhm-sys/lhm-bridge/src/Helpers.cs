namespace lhwm_bridge;

using System;
using System.Runtime.InteropServices;

/// <summary>
/// Pointer to a null terminated collection of UTF-8 encoded bytes
/// stored in un-managed memory
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public readonly struct Utf8Ptr : IDisposable
{
    /// <summary>
    /// Pointer to the actual data
    /// </summary>
    public readonly IntPtr data;

    public static Utf8Ptr Null = new Utf8Ptr(null);


    /// <summary>
    /// Create a UTF8 pointer 
    /// </summary>
    /// <param name="value"></param>
    public Utf8Ptr(string? value)
    {
        // Value is null or empty
        if (string.IsNullOrEmpty(value))
        {
            data = IntPtr.Zero;
            return;
        }


        // Encode the value as null terminated bytes
        byte[] utf8Bytes = System.Text.Encoding.UTF8.GetBytes(value + '\0');

        // Allocate and copy the bytes into un-managed memory
        IntPtr ptr = Marshal.AllocHGlobal(utf8Bytes.Length);
        Marshal.Copy(utf8Bytes, 0, ptr, utf8Bytes.Length);

        data = ptr;
    }

    /// <summary>
    /// Free the pointer
    /// </summary>
    public void Dispose()
    {
        if (data == IntPtr.Zero)
        {
            return;
        }

        Marshal.FreeHGlobal(data);
    }
}