; Iris Studio - Windows Inno Setup Production Installer Script
; Defines packaging, file associations, desktop icons, and uninstall hooks.

#define MyAppName "Iris Studio"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Iris Studio Open Source Team"
#define MyAppURL "https://github.com/iris-studio/iris-studio"
#define MyAppExeName "iris_studio.exe"

[Setup]
AppId={{5F7C32B1-4A59-4D88-B590-7F892B19D33A}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist\windows
OutputBaseFilename=IrisStudio-Setup-{#MyAppVersion}-x64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associateproj"; Description: "Associate Iris Studio Projects (.irisproj)"; GroupDescription: "File Associations:"
Name: "associatelut"; Description: "Associate 3D Color LUT files (.cube)"; GroupDescription: "File Associations:"; Flags: unchecked

[Files]
Source: "..\..\apps\flutter\build\windows\x64\runner\Release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\apps\flutter\build\windows\x64\runner\Release\*.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\apps\flutter\build\windows\x64\runner\Release\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\target\release\iris_ffi.dll"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; File association for .irisproj
Root: HKA; Subkey: "Software\Classes\.irisproj"; ValueType: string; ValueName: ""; ValueData: "IrisStudio.Project"; Flags: uninsdeletevalue; Tasks: associateproj
Root: HKA; Subkey: "Software\Classes\IrisStudio.Project"; ValueType: string; ValueName: ""; ValueData: "Iris Studio Project File"; Flags: uninsdeletekey; Tasks: associateproj
Root: HKA; Subkey: "Software\Classes\IrisStudio.Project\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: associateproj
Root: HKA; Subkey: "Software\Classes\IrisStudio.Project\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associateproj

; File association for .cube
Root: HKA; Subkey: "Software\Classes\.cube"; ValueType: string; ValueName: ""; ValueData: "IrisStudio.Lut"; Flags: uninsdeletevalue; Tasks: associatelut
Root: HKA; Subkey: "Software\Classes\IrisStudio.Lut"; ValueType: string; ValueName: ""; ValueData: "3D Color Grading LUT File"; Flags: uninsdeletekey; Tasks: associatelut
Root: HKA; Subkey: "Software\Classes\IrisStudio.Lut\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associatelut

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
