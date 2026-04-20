#define AppName "Robot Control & Serial Debug Suite"

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

#ifndef AppExeName
  #define AppExeName "robot_control_rust.exe"
#endif

#ifndef ProjectRoot
  #define ProjectRoot "."
#endif

#ifndef StageDir
  #define StageDir AddBackslash(ProjectRoot) + "dist\\windows-x64\\stage"
#endif

#ifndef OutputDir
  #define OutputDir AddBackslash(ProjectRoot) + "dist\\windows-x64\\installer"
#endif

[Setup]
AppId={{8CC71A6F-8B37-4E20-8BC2-AE0684F95E1D}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=Robot Control Team
AppPublisherURL=https://example.local/robot-control
DefaultDirName={autopf}\Robot Control Suite
DefaultGroupName=Robot Control Suite
DisableProgramGroupPage=yes
OutputDir={#OutputDir}
OutputBaseFilename=RobotControlSuite_{#AppVersion}_x64_Setup
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
UninstallDisplayIcon={app}\{#AppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}";

[Files]
Source: "{#StageDir}\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\rust_tools_suite.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\help_index.html"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#ProjectRoot}\ARCHITECTURE_AND_USAGE.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Robot Control Suite"; Filename: "{app}\{#AppExeName}"
Name: "{autoprograms}\Rust Tools Suite"; Filename: "{app}\rust_tools_suite.exe"
Name: "{autodesktop}\Robot Control Suite"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,Robot Control Suite}"; Flags: nowait postinstall skipifsilent
