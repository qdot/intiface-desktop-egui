#define Configuration GetEnv('CONFIGURATION')
#if Configuration == ""
#define Configuration "Release"
#endif

#define Version GetEnv('BUILD_VERSION')
#if Version == ""
#define Version "x.x.x.x"
#endif

[Setup]
AppName=Intiface Desktop
AppVersion={#Version}
AppPublisher=Nonpolynomial Labs, LLC
AppPublisherURL=www.buttplug.io
PrivilegesRequired=lowest
AlwaysUsePersonalGroup=yes
AppId={{0db5a4c3-d234-4923-a443-b5f63c683dbb}
SetupIconFile=icons\intiface-desktop-logo.ico
WizardImageFile=icons\intiface-desktop-logo.bmp
WizardSmallImageFile=icons\intiface-desktop-logo.bmp
DefaultDirName={userappdata}\IntifaceDesktopRust
UninstallDisplayIcon=icons\intiface-desktop-logo.ico
Compression=lzma2
SolidCompression=yes
OutputBaseFilename=intiface-desktop-installer
OutputDir=.\installer
LicenseFile=LICENSE

[Dirs]
Name: "{localappdata}\IntifaceDesktopRust"

[Files]
Source: "target\release\intiface-desktop.exe"; DestDir: "{app}"
Source: "..\intiface-cli-rs\target\release\intiface-cli.exe"; DestDir: "{app}\engine"
Source: "Readme.md"; DestDir: "{app}"; DestName: "Readme.txt"
Source: "LICENSE"; DestDir: "{app}"; DestName: "License.txt"

[Icons]
Name: "{userprograms}\Intiface Desktop Rust"; Filename: "{app}\intiface-desktop.exe"

// [Run]
// Filename: "{app}\Readme.txt"; Description: "View the README file"; Flags: postinstall shellexec unchecked

[Code]

// Uninstall on install code taken from https://stackoverflow.com/a/2099805/4040754
////////////////////////////////////////////////////////////////////
function GetUninstallString(): String;
var
  sUnInstPath: String;
  sUnInstallString: String;
begin
  sUnInstPath := ExpandConstant('Software\Microsoft\Windows\CurrentVersion\Uninstall\{#emit SetupSetting("AppId")}_is1');
  sUnInstallString := '';
  if not RegQueryStringValue(HKLM, sUnInstPath, 'UninstallString', sUnInstallString) then
    RegQueryStringValue(HKCU, sUnInstPath, 'UninstallString', sUnInstallString);
  Result := sUnInstallString;
end;


/////////////////////////////////////////////////////////////////////
function IsUpgrade(): Boolean;
begin
  Result := (GetUninstallString() <> '');
end;


/////////////////////////////////////////////////////////////////////
function UnInstallOldVersion(): Integer;
var
  sUnInstallString: String;
  iResultCode: Integer;
begin
// Return Values:
// 1 - uninstall string is empty
// 2 - error executing the UnInstallString
// 3 - successfully executed the UnInstallString

  // default return value
  Result := 0;

  // get the uninstall string of the old app
  sUnInstallString := GetUninstallString();
  if sUnInstallString <> '' then begin
    sUnInstallString := RemoveQuotes(sUnInstallString);
    if Exec(sUnInstallString, '/SILENT /NORESTART /SUPPRESSMSGBOXES','', SW_HIDE, ewWaitUntilTerminated, iResultCode) then
      Result := 3
    else
      Result := 2;
  end else
    Result := 1;
end;

/////////////////////////////////////////////////////////////////////
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep=ssInstall) then
  begin
    if (IsUpgrade()) then
    begin
      UnInstallOldVersion();
    end;
  end;
end;
