# LHM Service

This is a windows service that exposes a named pipe to allow user-land consumers to 
request the hardware information from the service using Libre Hardware Monitor internally.

Can be used to determine CPU and GPU temperatures in user-land without requiring your app
be launched as admin (Installing the service still requires admin rights)

Requires .NET SDK 8.0 to build, you can install this through winget using:

```sh
winget install Microsoft.DotNet.SDK.8
```

