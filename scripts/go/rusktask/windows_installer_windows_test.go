//go:build windows

package main

import (
	"fmt"
	"path/filepath"
	"strings"
	"testing"
)

func TestRenderWindowsInstallerNSISUsesNativePaths(t *testing.T) {
	template := strings.Join([]string{
		`Name "Robot Control Suite ${VERSION}"`,
		`OutFile "robot_control_suite_${VERSION}_windows_x64-setup.exe"`,
		`Section "Install"`,
		`    SetOutPath "$INSTDIR"`,
		`    File "target\release\robot_control_rust.exe"`,
		`    File "target\release\rust_tools_suite.exe"`,
		`SectionEnd`,
	}, "\n")
	repoRoot := `D:\a\robot_ctrl_rust_app\robot_ctrl_rust_app`
	stageDir := filepath.Join(repoRoot, `release_artifacts\windows-x64\stage`)
	outputDir := filepath.Join(repoRoot, `release_artifacts\windows-x64\installer`)

	rendered := renderWindowsInstallerNSIS(template, "0.1.9", stageDir, outputDir)

	if strings.Contains(rendered, "D:/a/") {
		t.Fatalf("rendered NSIS script contains POSIX-style GitHub Actions path:\n%s", rendered)
	}

	expectedMainFile := fmt.Sprintf(`File %q`, filepath.Clean(filepath.Join(stageDir, "robot_control_rust.exe")))
	if !strings.Contains(rendered, expectedMainFile) {
		t.Fatalf("rendered NSIS script missing main executable File line %q:\n%s", expectedMainFile, rendered)
	}

	expectedSuiteFile := fmt.Sprintf(`File %q`, filepath.Clean(filepath.Join(stageDir, "rust_tools_suite.exe")))
	if !strings.Contains(rendered, expectedSuiteFile) {
		t.Fatalf("rendered NSIS script missing suite executable File line %q:\n%s", expectedSuiteFile, rendered)
	}

	expectedOutFile := fmt.Sprintf(
		`OutFile %q`,
		filepath.Clean(filepath.Join(outputDir, "robot_control_suite_0.1.9_windows_x64-setup.exe")),
	)
	if !strings.Contains(rendered, expectedOutFile) {
		t.Fatalf("rendered NSIS script missing installer OutFile line %q:\n%s", expectedOutFile, rendered)
	}

	for _, stale := range []string{
		`File "target\release\robot_control_rust.exe"`,
		`File "target\release\rust_tools_suite.exe"`,
		`OutFile "robot_control_suite_${VERSION}_windows_x64-setup.exe"`,
	} {
		if strings.Contains(rendered, stale) {
			t.Fatalf("rendered NSIS script still contains stale template token %q:\n%s", stale, rendered)
		}
	}
}
