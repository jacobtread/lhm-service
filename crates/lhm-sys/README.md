# LHM Sys

> Wrapper around Libre Hardware Monitor to request a list of hardware

System library providing rust bindings to Libre Hardware Monitor through a DLL generated
through a thin C# wrapper over Libre Hardware Monitor

Requires .NET SDK 8.0 to build, you can install this through winget using:

```
winget install Microsoft.DotNet.SDK.8
```

By default static linking will be used, you can change this by disabling default features and specifying the "dylib" feature. When you use this feature you must 
ensure that the lhm-bridge.dll is present in the same directory as your executable
when you run it. You can find `lhm-bridge.dll` in `target/{PROFILE}/build/lhm-sys-{HASH}/out/lhm-bridge/lhm-bridge.dll`