namespace lhwm_bridge;

using System;
using System.Runtime.InteropServices;


/// <summary>
/// FFI safe array type for a shared array 
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public struct SharedFfiArray<T>
{
    /// <summary>
    /// Length of the array
    /// </summary>
    public int length;

    /// <summary>
    /// Actual pointer data to the array
    /// </summary>
    private SharedFfiArrayPtr ptr;

    public SharedFfiArray(T[] array)
    {
        length = array.Length;
        var ptr = GCHandle.Alloc(array, GCHandleType.Pinned);

        this.ptr = new SharedFfiArrayPtr
        {
            data = ptr.AddrOfPinnedObject(),
            handle = GCHandle.ToIntPtr(ptr),
        };
    }

    public void Dispose()
    {
        ptr.Dispose();
    }
}


/// <summary>
/// FFI safe array type for a shared array pointer
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public struct SharedFfiArrayPtr
{
    /// <summary>
    /// Pointer to the data for the array
    /// </summary>
    public IntPtr data;

    /// <summary>
    /// Pointer to the handle for freeing the array
    /// </summary>
    public IntPtr handle;


    public void Dispose()
    {
        var gcHandle = GCHandle.FromIntPtr(handle);
        gcHandle.Free();
    }
}