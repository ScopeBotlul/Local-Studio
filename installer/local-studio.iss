; Inno Setup 7.1+. The shipped setup follows Windows at startup.
#ifndef AppVersion
  #error AppVersion must be supplied by scripts/build-installer.ps1
#endif
#ifndef ProjectRoot
  #error ProjectRoot must be supplied by scripts/build-installer.ps1
#endif
#ifndef TestBuild
  #define ProjectExtension ".localstudio"
  #define ProjectProgId "LocalStudio.Project"
  #define SetupId "de.localstudio.desktop"
  #define SetupName "Local Studio"
  #define SetupStyle "modern dynamic windows11"
  #define OutputName "Local-Studio-" + AppVersion + "-hub-setup"
#else
  #define ProjectExtension ".localstudio-test-" + TestBuild
  #define ProjectProgId "LocalStudio.Test." + TestBuild + ".Project"
  #define SetupId "de.localstudio.installer-test." + TestBuild
  #define SetupName "Local Studio Installer Test " + TestBuild
  #ifndef SetupStyle
    #define SetupStyle "modern dynamic windows11"
  #endif
  #define OutputName "installer-test-" + TestBuild
#endif

[Setup]
AppId={#SetupId}
AppName={#SetupName}
AppVersion={#AppVersion}
AppPublisher=ScopeBotlul
AppPublisherURL=https://github.com/ScopeBotlul/Local-Studio
DefaultDirName={localappdata}\{#SetupName}
DefaultGroupName={#SetupName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
WizardStyle={#SetupStyle}
SetupIconFile={#ProjectRoot}\src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\local-studio.exe
OutputDir={#ProjectRoot}\src-tauri\target\release\bundle\inno
OutputBaseFilename={#OutputName}
Compression=lzma2
SolidCompression=yes
ChangesAssociations=yes
CloseApplications=no
RestartApplications=no
DisableWelcomePage=no
LanguageDetectionMethod=uilanguage
ShowLanguageDialog=auto
UninstallDisplayName={#SetupName}

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
german.DesktopShortcut=Desktop-Verknüpfung erstellen
english.DesktopShortcut=Create a desktop shortcut
german.LaunchApp=Local Studio starten
english.LaunchApp=Launch Local Studio
german.LegacySetup=Eine ältere NSIS-Installation von Local Studio wurde gefunden. Bitte deinstalliere sie zuerst über Windows > Installierte Apps. Wähle dabei NICHT das Löschen der App-Daten. Danach dieses Setup erneut starten. Modelle und portable Ordner nicht löschen.
english.LegacySetup=An older NSIS installation of Local Studio was found. Uninstall it first through Windows > Installed apps. Do NOT select removal of application data. Then run this setup again. Keep your models and portable folders.
german.CloseApp=Bitte Local Studio vollständig schließen und das Setup erneut versuchen.
english.CloseApp=Please close Local Studio completely and retry setup.
german.PortableFolder=Dieser Ordner enthält eine portable Version. Bitte einen anderen Installationsordner wählen. Portable Versionen werden durch Ersetzen ihrer Programmdateien aktualisiert.
english.PortableFolder=This folder contains a portable version. Choose another installation folder. Portable versions are updated by replacing their program files.
german.WebViewFailed=Microsoft Edge WebView2 konnte nicht installiert werden. Bitte die WebView2 Runtime von Microsoft installieren und dieses Setup erneut starten.
english.WebViewFailed=Microsoft Edge WebView2 could not be installed. Install the Microsoft WebView2 Runtime and run this setup again.

[Tasks]
Name: "desktopicon"; Description: "{cm:DesktopShortcut}"; Flags: unchecked

[Files]
Source: "{#ProjectRoot}\src-tauri\installed.marker"; DestDir: "{app}"; Flags: ignoreversion
#if defined(TestBuild) && defined(TestBinary)
Source: "{#TestBinary}"; DestDir: "{app}"; DestName: "local-studio.exe"; Flags: ignoreversion
#else
Source: "{#ProjectRoot}\src-tauri\target\release\local-studio.exe"; DestDir: "{app}"; Flags: ignoreversion
#endif
Source: "{#ProjectRoot}\.tools\webview2\MicrosoftEdgeWebview2Setup.exe"; Flags: dontcopy

Source: "{#ProjectRoot}\src-tauri\target\release\image-runtime\*"; DestDir: "{app}\image-runtime"; Flags: ignoreversion

[Icons]
#ifndef TestBuild
Name: "{autoprograms}\Local Studio"; Filename: "{app}\local-studio.exe"
Name: "{autodesktop}\Local Studio"; Filename: "{app}\local-studio.exe"; Tasks: desktopicon
#endif

[Registry]
Root: HKCU; Subkey: "Software\Classes\{#ProjectExtension}"; ValueType: string; ValueData: "{#ProjectProgId}"; Flags: createvalueifdoesntexist uninsdeletekeyifempty
Root: HKCU; Subkey: "Software\Classes\{#ProjectExtension}\OpenWithProgids"; ValueType: string; ValueName: "{#ProjectProgId}"; ValueData: ""; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKCU; Subkey: "Software\Classes\{#ProjectProgId}"; ValueType: string; ValueData: "Local Studio Project"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\{#ProjectProgId}\DefaultIcon"; ValueType: string; ValueData: """{app}\local-studio.exe"",0"
Root: HKCU; Subkey: "Software\Classes\{#ProjectProgId}\shell\open\command"; ValueType: string; ValueData: """{app}\local-studio.exe"" ""%1"""

[Run]
#ifndef TestBuild
Filename: "{app}\local-studio.exe"; Description: "{cm:LaunchApp}"; Flags: nowait postinstall skipifsilent unchecked
#endif

[Code]
const
  LegacyKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\Local Studio';
  WebViewKey = 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';

function HasLegacySetup: Boolean;
begin
#ifdef TestLegacy
  Result := True;
#elif defined(TestBuild)
  Result := False;
#else
  Result := RegKeyExists(HKCU32, LegacyKey) or RegKeyExists(HKCU64, LegacyKey) or
    RegKeyExists(HKLM32, LegacyKey) or RegKeyExists(HKLM64, LegacyKey);
#endif
end;

function InitializeSetup: Boolean;
begin
  Result := not HasLegacySetup;
  if not Result then begin
    Log('Local Studio: legacy NSIS installation blocked; no files changed.');
    SuppressibleMsgBox(CustomMessage('LegacySetup'), mbInformation, MB_OK, IDOK);
  end;
end;

procedure InitializeWizard;
begin
  if IsDarkInstallMode then Log('Local Studio installer theme: dark')
  else Log('Local Studio installer theme: light');
end;

function InitializeUninstall: Boolean;
begin
  Result := True;
  if IsDarkInstallMode then Log('Local Studio uninstaller theme: dark')
  else Log('Local Studio uninstaller theme: light');
end;

function RuntimeAt(Root: Integer): Boolean;
var Version: String;
begin
  Result := RegQueryStringValue(Root, WebViewKey, 'pv', Version) and
    (Version <> '') and (Version <> '0.0.0.0');
end;

function HasWebView: Boolean;
begin
  Result := RuntimeAt(HKCU32) or RuntimeAt(HKCU64) or RuntimeAt(HKLM32) or RuntimeAt(HKLM64);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var Code: Integer; Target: String; Stream: TFileStream;
begin
  Result := '';
  if HasLegacySetup then begin Result := CustomMessage('LegacySetup'); exit; end;
  if FileExists(ExpandConstant('{app}\portable.marker')) then begin
    Result := CustomMessage('PortableFolder'); exit;
  end;
  Target := ExpandConstant('{app}\local-studio.exe');
  if FileExists(Target) then begin
    try
      Stream := TFileStream.Create(Target, fmOpenReadWrite or fmShareExclusive);
      Stream.Free;
    except
      Result := CustomMessage('CloseApp'); exit;
    end;
  end;
  if not HasWebView then begin
    ExtractTemporaryFile('MicrosoftEdgeWebview2Setup.exe');
    if not Exec(ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe'), '/silent /install',
      '', SW_HIDE, ewWaitUntilTerminated, Code) then begin
      Result := CustomMessage('WebViewFailed'); exit;
    end;
    if (Code <> 0) and (Code <> 3010) then begin Result := CustomMessage('WebViewFailed'); exit; end;
    if not HasWebView then begin Result := CustomMessage('WebViewFailed'); exit; end;
    NeedsRestart := Code = 3010;
  end;
end;

// No recursive UninstallDelete rules: user data, models and added files remain.

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Association: String;
begin
  if CurUninstallStep = usUninstall then
    if RegQueryStringValue(HKCU, 'Software\Classes\{#ProjectExtension}', '', Association) then
      if Association = '{#ProjectProgId}' then
        RegDeleteValue(HKCU, 'Software\Classes\{#ProjectExtension}', '');
end;
