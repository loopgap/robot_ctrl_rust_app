; NSIS installer script for Robot Control Rust (x64)
; All values can be overridden by /D switches in makensis.

!include "MUI2.nsh"
!include "x64.nsh"

!ifndef APP_NAME
  !define APP_NAME "Robot Control Suite"
!endif

!ifndef APP_EXE
  !define APP_EXE "robot_control_rust.exe"
!endif

!ifndef APP_VERSION
  !define APP_VERSION "0.1.0"
!endif

!ifndef APP_PUBLISHER
  !define APP_PUBLISHER "Robot Control Team"
!endif

!ifndef STAGE_DIR
  !define STAGE_DIR "..\\dist\\windows-x64\\stage"
!endif

!ifndef OUTPUT_DIR
  !define OUTPUT_DIR "..\\dist\\windows-x64\\installer"
!endif

Name "${APP_NAME} ${APP_VERSION}"
OutFile "${OUTPUT_DIR}\\RobotControlSuite_${APP_VERSION}_x64_Setup.exe"
InstallDir "$PROGRAMFILES64\\Robot Control Suite"
InstallDirRegKey HKLM "Software\\Robot Control Suite" "InstallDir"
RequestExecutionLevel admin
Unicode True

VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey "ProductName" "${APP_NAME}"
VIAddVersionKey "FileDescription" "${APP_NAME} Installer"
VIAddVersionKey "CompanyName" "${APP_PUBLISHER}"
VIAddVersionKey "FileVersion" "${APP_VERSION}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"

!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\\Contrib\\Graphics\\Icons\\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\\Contrib\\Graphics\\Icons\\modern-uninstall.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Install"
  ${IfNot} ${RunningX64}
    MessageBox MB_ICONSTOP "This installer supports x64 Windows only."
    Abort
  ${EndIf}

  SetOutPath "$INSTDIR"
  File "${STAGE_DIR}\\${APP_EXE}"
  File "${STAGE_DIR}\\ARCHITECTURE_AND_USAGE.md"

  WriteRegStr HKLM "Software\\Robot Control Suite" "InstallDir" "$INSTDIR"

  CreateDirectory "$SMPROGRAMS\\Robot Control Suite"
  CreateShortcut "$SMPROGRAMS\\Robot Control Suite\\Robot Control Suite.lnk" "$INSTDIR\\${APP_EXE}"
  CreateShortcut "$DESKTOP\\Robot Control Suite.lnk" "$INSTDIR\\${APP_EXE}"

  WriteUninstaller "$INSTDIR\\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\\${APP_EXE}"
  Delete "$INSTDIR\\ARCHITECTURE_AND_USAGE.md"
  Delete "$INSTDIR\\Uninstall.exe"

  Delete "$SMPROGRAMS\\Robot Control Suite\\Robot Control Suite.lnk"
  RMDir "$SMPROGRAMS\\Robot Control Suite"
  Delete "$DESKTOP\\Robot Control Suite.lnk"

  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\\Robot Control Suite"
SectionEnd
