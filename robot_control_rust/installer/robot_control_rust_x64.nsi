!ifndef APP_NAME
  !define APP_NAME "Robot Control & Serial Debug Suite"
!endif

!ifndef APP_EXE
  !define APP_EXE "robot_control_rust.exe"
!endif

!ifndef APP_VERSION
  !define APP_VERSION "0.1.0"
!endif

!ifndef STAGE_DIR
  !define STAGE_DIR ".\\dist\\windows-x64\\stage"
!endif

!ifndef OUTPUT_DIR
  !define OUTPUT_DIR ".\\dist\\windows-x64\\installer"
!endif

Name "${APP_NAME} ${APP_VERSION}"
OutFile "${OUTPUT_DIR}\\RobotControlSuite_${APP_VERSION}_x64_Setup.exe"
InstallDir "$PROGRAMFILES64\\Robot Control Suite"
InstallDirRegKey HKLM "Software\\Robot Control Suite" "InstallDir"
RequestExecutionLevel admin
Unicode True

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File "${STAGE_DIR}\\${APP_EXE}"
  File "${STAGE_DIR}\\rust_tools_suite.exe"
  File "${STAGE_DIR}\\help_index.html"
  File "${STAGE_DIR}\\ARCHITECTURE_AND_USAGE.md"

  WriteRegStr HKLM "Software\\Robot Control Suite" "InstallDir" "$INSTDIR"

  CreateDirectory "$SMPROGRAMS\\Robot Control Suite"
  CreateShortcut "$SMPROGRAMS\\Robot Control Suite\\Robot Control Suite.lnk" "$INSTDIR\\${APP_EXE}"
  CreateShortcut "$SMPROGRAMS\\Robot Control Suite\\Rust Tools Suite.lnk" "$INSTDIR\\rust_tools_suite.exe"
  CreateShortcut "$DESKTOP\\Robot Control Suite.lnk" "$INSTDIR\\${APP_EXE}"

  WriteUninstaller "$INSTDIR\\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\\${APP_EXE}"
  Delete "$INSTDIR\\rust_tools_suite.exe"
  Delete "$INSTDIR\\help_index.html"
  Delete "$INSTDIR\\ARCHITECTURE_AND_USAGE.md"
  Delete "$INSTDIR\\Uninstall.exe"

  Delete "$SMPROGRAMS\\Robot Control Suite\\Robot Control Suite.lnk"
  Delete "$SMPROGRAMS\\Robot Control Suite\\Rust Tools Suite.lnk"
  RMDir "$SMPROGRAMS\\Robot Control Suite"
  Delete "$DESKTOP\\Robot Control Suite.lnk"

  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\\Robot Control Suite"
SectionEnd
