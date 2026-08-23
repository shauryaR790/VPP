; v++ Windows installer  -  built in CI (Release workflow)
#ifndef MyAppVersion
  #define MyAppVersion "0.5.0"
#endif

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=v++
AppVersion={#MyAppVersion}
AppPublisher=vpp-lang
AppPublisherURL=https://github.com/shauryaR790/VPP
AppSupportURL=https://github.com/shauryaR790/VPP/issues
AppUpdatesURL=https://github.com/shauryaR790/VPP/releases
DefaultDirName={autopf}\vpp
DefaultGroupName=v++
DisableProgramGroupPage=yes
LicenseFile=..\staging\LICENSE
OutputDir=..\output
OutputBaseFilename=vpp-{#MyAppVersion}-setup
UninstallDisplayIcon={app}\vpp.exe
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
Compression=lzma2
SolidCompression=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut to run hello.vpp"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "..\staging\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\v++ - Run hello.vpp"; Filename: "{app}\vpp.exe"; Parameters: "run examples\hello.vpp"; WorkingDir: "{app}"
Name: "{group}\v++ - Open install folder"; Filename: "{app}"
Name: "{group}\Uninstall v++"; Filename: "{uninstallexe}"
Name: "{autodesktop}\v++"; Filename: "{app}\vpp.exe"; Parameters: "run examples\hello.vpp"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\vpp.exe"; Parameters: "run examples\hello.vpp"; Description: "Run the hello.vpp example"; Flags: postinstall nowait skipifsilent

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddAppPath
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\llvm\bin"; Check: NeedsAddLlvmPath

[Code]
function NeedsAddPath(Param: string): Boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

function NeedsAddAppPath: Boolean;
begin
  Result := NeedsAddPath(ExpandConstant('{app}'));
end;

function NeedsAddLlvmPath: Boolean;
begin
  Result := NeedsAddPath(ExpandConstant('{app}') + '\llvm\bin');
end;
