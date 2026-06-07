package main

import "testing"

func TestValidateReleaseNotesReleaseModeDoesNotRequireFakeCiOrAssetChecks(t *testing.T) {
	content := `# v0.2.0

## Highlights
- Added simulation lab.

## Fixes
- Hardened release validation.

## Verification
- [x] ./scripts/windows/task.ps1 preflight
- [ ] Local release artifact smoke checks completed
- [ ] GitHub release workflow verifies uploaded artifacts and checksums
`

	if err := validateReleaseNotes(content, "release"); err != nil {
		t.Fatalf("release notes should validate without checked CI/assets claims: %v", err)
	}
}

func TestValidateReleaseNotesReleaseModeRequiresCheckedPreflight(t *testing.T) {
	content := `# v0.2.0

## Highlights
- Added simulation lab.

## Fixes
- Hardened release validation.

## Verification
- [ ] ./scripts/windows/task.ps1 preflight
- [ ] Local release artifact smoke checks completed
- [ ] GitHub release workflow verifies uploaded artifacts and checksums
`

	if err := validateReleaseNotes(content, "release"); err == nil {
		t.Fatal("release mode should require checked preflight")
	}
}
