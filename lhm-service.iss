; Define your application information
[Setup]
AppName=Hardware Monitoring Service
AppVersion=1.0
DefaultDirName={pf}\LibreHardwareMonitorService
DefaultGroupName=LibreHardwareMonitorService
OutputDir=.
OutputBaseFilename=lhm-setup
Compression=lzma
SolidCompression=yes

; Specify the application files
[Files]
Source: ".\target\release\lhm-service.exe"; DestDir: "{app}"; Flags: ignoreversion

; Create the service
[Run]
Filename: "{app}\lhm-service.exe"; Parameters: "create"; StatusMsg: "Installing service..."; Flags: runhidden waituntilterminated

; Start the service
[Run]
Filename: "{app}\lhm-service.exe"; Parameters: "start"; StatusMsg: "Starting service..."; Flags: runhidden waituntilterminated

; Define the uninstall procedure
[UninstallRun]
Filename: "{app}\lhm-service.exe"; Parameters: "stop"; StatusMsg: "Stopping service..."; Flags: runhidden waituntilterminated
Filename: "{app}\lhm-service.exe"; Parameters: "delete"; StatusMsg: "Deleting service..."; Flags: runhidden waituntilterminated

; Optional: define the messages for the user
[Messages]
SetupMessage=LHM service is being installed...
UninstallMessage=LHM service is being uninstalled...
