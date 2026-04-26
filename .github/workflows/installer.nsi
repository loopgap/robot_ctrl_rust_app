!include "MUI2.nsh"
Name "Robot Control Suite ${VERSION}"
OutFile "robot_control_suite_${VERSION}_windows_x64-setup.exe"
InstallDir "$PROGRAMFILES64\Robot Control Suite"
RequestExecutionLevel admin
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
Section "Install"
    SetOutPath "$INSTDIR"
    File "target\release\robot_control_rust.exe"
    File "target\release\rust_tools_suite.exe"
    CreateDirectory "$SMPROGRAMS\Robot Control Suite"
    CreateShortcut "$SMPROGRAMS\Robot Control Suite\Robot Control.lnk" "$INSTDIR\robot_control_rust.exe"
    CreateShortcut "$SMPROGRAMS\Robot Control Suite\Tools Suite.lnk" "$INSTDIR\rust_tools_suite.exe"
    CreateShortcut "$SMPROGRAMS\Robot Control Suite\Uninstall.lnk" "$INSTDIR\uninstall.exe"
    WriteUninstaller "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" "DisplayName" "Robot Control Suite ${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" "Publisher" "Robot Control"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" "NoRepair" 1
SectionEnd
Section "Uninstall"
    Delete "$INSTDIR\robot_control_rust.exe"
    Delete "$INSTDIR\rust_tools_suite.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    Delete "$SMPROGRAMS\Robot Control Suite\Robot Control.lnk"
    Delete "$SMPROGRAMS\Robot Control Suite\Tools Suite.lnk"
    Delete "$SMPROGRAMS\Robot Control Suite\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Robot Control Suite"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite"
SectionEnd