
using System.Runtime.InteropServices;


/// <summary>
/// FFI safe array type for a shared array 
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public struct SharedFfiArray<T> : IDisposable where T : struct, IDisposable
{
    /// <summary>
    /// Length of the array
    /// </summary>
    public int length;

    /// <summary>
    /// Actual pointer data to the array
    /// </summary>
    private SharedFfiArrayPtr<T> ptr;

    public SharedFfiArray(T[] array)
    {
        length = array.Length;
        ptr = new SharedFfiArrayPtr<T>(array);
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
public struct SharedFfiArrayPtr<T> : IDisposable where T : struct, IDisposable
{
    /// <summary>
    /// Pointer to the data for the array
    /// </summary>
    private IntPtr data;

    /// <summary>
    /// Pointer to the handle for freeing the array
    /// </summary>
    private IntPtr handle;

    public SharedFfiArrayPtr(T[] array)
    {
        var ptr = GCHandle.Alloc(array, GCHandleType.Pinned);
        data = ptr.AddrOfPinnedObject();
        handle = GCHandle.ToIntPtr(ptr);
    }

    public void Dispose()
    {
        var gcHandle = GCHandle.FromIntPtr(handle);
        gcHandle.Free();
    }
}