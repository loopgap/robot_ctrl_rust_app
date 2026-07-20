; ─────────────────────────────────────────────────────────────
; Robot Control Suite — Windows Installer (NSIS)
;
; NOTE: This is a build script, NOT a GitHub Actions workflow.
;       Placed under scripts/nsis/ for semantic clarity.
;
; Usage:  makensis /DVERSION=0.2.0 installer.nsi
; CI:     Called via `go run . package-windows-installer` in release.yml
; ─────────────────────────────────────────────────────────────

!include "MUI2.nsh"

; ── Metadata ──────────────────────────────────────────────
Name "Robot Control Suite ${VERSION}"
OutFile "robot_control_suite_${VERSION}_windows_x64-setup.exe"
InstallDir "$PROGRAMFILES64\Robot Control Suite"
InstallDirRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" "InstallLocation"
RequestExecutionLevel admin
BrandingText "Robot Control Suite v${VERSION}"

; ── Version info embedded in EXE ──────────────────────────
VIProductVersion "${VERSION}.0.0"
VIAddVersionKey "ProductName" "Robot Control Suite"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "Robot Control Suite Installer"
VIAddVersionKey "LegalCopyright" "GPL-3.0-only"

; ── Modern UI pages ───────────────────────────────────────
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; ── Install section ───────────────────────────────────────
Section "Install" SecInstall
    SetOutPath "$INSTDIR"

    ; Binaries
    File "target\release\robot_control_rust.exe"
    File "target\release\rust_tools_suite.exe"

    ; Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\Robot Control Suite"
    CreateShortcut "$SMPROGRAMS\Robot Control Suite\Robot Control.lnk" "$INSTDIR\robot_control_rust.exe"
    CreateShortcut "$SMPROGRAMS\Robot Control Suite\Tools Suite.lnk" "$INSTDIR\rust_tools_suite.exe"
    CreateShortcut "$SMPROGRAMS\Robot Control Suite\Uninstall.lnk" "$INSTDIR\uninstall.exe"

    ; Uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Add/Remove Programs registry keys
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "DisplayName" "Robot Control Suite ${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "Publisher" "Robot Control Team"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "DisplayVersion" "${VERSION}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite" \
        "NoRepair" 1
SectionEnd

; ── Uninstall section ─────────────────────────────────────
Section "Uninstall"
    ; Remove binaries
    Delete "$INSTDIR\robot_control_rust.exe"
    Delete "$INSTDIR\rust_tools_suite.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    ; Remove Start Menu shortcuts
    Delete "$SMPROGRAMS\Robot Control Suite\Robot Control.lnk"
    Delete "$SMPROGRAMS\Robot Control Suite\Tools Suite.lnk"
    Delete "$SMPROGRAMS\Robot Control Suite\Uninstall.lnk"
    RMDir "$SMPROGRAMS\Robot Control Suite"

    ; Remove registry keys
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\RobotControlSuite"
SectionEnd
