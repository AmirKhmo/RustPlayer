; RustPlayer installer (Inno Setup 6). Built by build-windows.ps1 after dist\app is staged.
#define AppVer "0.3.1"

[Setup]
AppId={{8C3F2B71-5D4A-4E8B-9F61-2A7D3C9E1B40}
AppName=RustPlayer
AppVersion={#AppVer}
AppPublisher=RustPlayer
DefaultDirName={autopf}\RustPlayer
DefaultGroupName=RustPlayer
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=RustPlayer-Setup-{#AppVer}
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\rustplayer.exe
Compression=lzma2/ultra64
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
ChangesAssociations=yes
WizardStyle=modern
PrivilegesRequired=admin

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "assoc"; Description: "Add RustPlayer to 'Open with' for video and audio files"; GroupDescription: "File types:"

[Files]
Source: "..\dist\app\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; Install Vazirmatn system-wide so mpv/libass can always find it for Persian subtitles
Source: "..\dist\app\fonts\Vazirmatn-Regular.ttf"; DestDir: "{autofonts}"; FontInstall: "Vazirmatn"; Flags: onlyifdoesntexist uninsneveruninstall skipifsourcedoesntexist

[Icons]
Name: "{group}\RustPlayer"; Filename: "{app}\rustplayer.exe"
Name: "{group}\Uninstall RustPlayer"; Filename: "{uninstallexe}"
Name: "{autodesktop}\RustPlayer"; Filename: "{app}\rustplayer.exe"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\RustPlayer.Media"; ValueType: string; ValueData: "RustPlayer media file"; Flags: uninsdeletekey; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\RustPlayer.Media\DefaultIcon"; ValueType: string; ValueData: "{app}\rustplayer.exe,0"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\RustPlayer.Media\shell\open\command"; ValueType: string; ValueData: """{app}\rustplayer.exe"" ""%1"""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "RustPlayer"; Flags: uninsdeletekey; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\shell\open\command"; ValueType: string; ValueData: """{app}\rustplayer.exe"" ""%1"""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.mkv\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.mp4\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.avi\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.mov\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.webm\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.flv\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.wmv\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.ts\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.m2ts\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.mpg\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.mpeg\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.m4v\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.3gp\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.vob\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.mp3\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.flac\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.wav\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.ogg\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.m4a\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.opus\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.aac\OpenWithProgids"; ValueType: string; ValueName: "RustPlayer.Media"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".mkv"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".mp4"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".avi"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".mov"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".webm"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".flv"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".wmv"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".ts"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".m2ts"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".mpg"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".mpeg"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".m4v"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".3gp"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".vob"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".mp3"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".flac"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".wav"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".ogg"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".m4a"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".opus"; ValueData: ""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\rustplayer.exe\SupportedTypes"; ValueType: string; ValueName: ".aac"; ValueData: ""; Tasks: assoc

[Run]
Filename: "{app}\rustplayer.exe"; Description: "{cm:LaunchProgram,RustPlayer}"; Flags: nowait postinstall skipifsilent
