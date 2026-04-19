package main

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

const cliVersion = "0.1.0-dev"

const (
	exitSuccess      = 0
	exitExecution    = 1
	exitUsage        = 2
	exitPrecondition = 3
)

type projectDef struct {
	Name        string
	DirRelPath  string
	CargoToml   string
	CargoLock   string
	ReleaseOnly bool
}

var allProjects = []projectDef{
	{
		Name:       "robot_control_rust",
		DirRelPath: filepath.FromSlash("robot_control_rust"),
		CargoToml:  filepath.FromSlash("robot_control_rust/Cargo.toml"),
		CargoLock:  filepath.FromSlash("robot_control_rust/Cargo.lock"),
	},
	{
		Name:       "rust_tools_suite",
		DirRelPath: filepath.FromSlash("rust_tools_suite"),
		CargoToml:  filepath.FromSlash("rust_tools_suite/Cargo.toml"),
		CargoLock:  filepath.FromSlash("rust_tools_suite/Cargo.lock"),
	},
}

var defaultAuditIgnores = []string{"RUSTSEC-2023-0071"}
var errStopWalk = errors.New("stop-walk")

type governanceConfig struct {
	Workspace workspacePolicy `json:"workspace"`
	Cleanup   cleanupPolicy   `json:"cleanup"`
}

type workspacePolicy struct {
	AllowedRootEntries        []string `json:"allowedRootEntries"`
	AllowedRootRegex          []string `json:"allowedRootRegex"`
	BlockedPathRegex          []string `json:"blockedPathRegex"`
	BlockedFixedRelativePaths []string `json:"blockedFixedRelativePaths"`
	BlockedGlobPatterns       []string `json:"blockedGlobPatterns"`
}

type cleanupPolicy struct {
	FixedRelativePaths     []string `json:"fixedRelativePaths"`
	GlobPatterns           []string `json:"globPatterns"`
	ProtectedRelativePaths []string `json:"protectedRelativePaths"`
}

type releaseStateSnapshot struct {
	LocalTags     []string
	RemoteTags    []string
	NoteMap       map[string]string
	NoteTags      []string
	LocalOnlyTags []string
	OrphanNotes   []string
	OrphanTags    []string
}

type releaseIndexRow struct {
	Major            int
	Minor            int
	Patch            int
	SuffixRank       int
	Suffix           string
	Version          string
	Tag              string
	LocalTagStatus   string
	RemoteTagStatus  string
	ReleaseNotesPath string
	ArchiveStatus    string
	ArchivePath      string
}

type semverTagInfo struct {
	Tag        string
	Version    string
	Major      int
	Minor      int
	Patch      int
	Suffix     string
	SuffixRank int
}

type githubReleaseAsset struct {
	ID   int64  `json:"id"`
	Name string `json:"name"`
}

type githubRelease struct {
	ID        int64                `json:"id"`
	HTMLURL   string               `json:"html_url"`
	UploadURL string               `json:"upload_url"`
	Assets    []githubReleaseAsset `json:"assets"`
}

func main() {
	os.Exit(run(os.Args[1:]))
}

func run(args []string) int {
	if len(args) == 0 {
		printUsage()
		return 0
	}

	switch args[0] {
	case "help", "-h", "--help":
		printUsage()
		return exitSuccess
	case "version":
		fmt.Println(cliVersion)
		return exitSuccess
	case "fmt":
		return runFmt(args[1:])
	case "clippy":
		return runClippy(args[1:])
	case "test":
		return runTest(args[1:])
	case "build":
		return runBuild(args[1:])
	case "doc":
		return runDoc(args[1:])
	case "audit":
		return runAudit(args[1:])
	case "check":
		return runCheck(args[1:])
	case "preflight":
		return runPreflight(args[1:])
	case "git-check":
		return runGitCheck(args[1:])
	case "rust-review":
		return runRustReview(args[1:])
	case "review":
		return runReview(args[1:])
	case "install-hooks":
		return runInstallHooks(args[1:])
	case "smart-bump":
		return runSmartBump(args[1:])
	case "smart-rollback":
		return runSmartRollback(args[1:])
	case "pr-helper":
		return runPRHelper(args[1:])
	case "release-sync":
		return runReleaseSync(args[1:])
	case "workflow-seal":
		return runWorkflowSeal(args[1:])
	case "workspace-cleanup":
		return runWorkspaceCleanup(args[1:])
	case "workspace-guard":
		return runWorkspaceGuard(args[1:])
	case "docs-bundle":
		return runDocsBundle(args[1:])
	case "release-publish":
		return runReleasePublish(args[1:])
	case "build-release-slim":
		return runBuildReleaseSlim(args[1:])
	case "package-windows-installer":
		return runPackageWindowsInstaller(args[1:])
	case "package-windows-assets":
		return runPackageWindowsAssets(args[1:])
	case "package-windows-portable-installer":
		return runPackageWindowsPortableInstaller(args[1:])
	case "update-release-index":
		return runUpdateReleaseIndexCommand(args[1:])
	case "release-notes":
		return runReleaseNotes(args[1:])
	default:
		fmt.Fprintf(os.Stderr, "unknown command: %s\n\n", args[0])
		printUsage()
		return exitUsage
	}
}

func runFmt(args []string) int {
	fs := flag.NewFlagSet("fmt", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	check := fs.Bool("check", false, "Only check formatting")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	for _, project := range allProjects {
		cargoArgs := []string{"fmt"}
		if *check {
			cargoArgs = append(cargoArgs, "--check")
		}
		cargoArgs = append(cargoArgs, "--manifest-path", filepath.Join(repoRoot, project.CargoToml))

		if err := runCommand(repoRoot, "cargo", cargoArgs, nil); err != nil {
			return exitExecution
		}
	}

	return exitSuccess
}

func runClippy(args []string) int {
	fs := flag.NewFlagSet("clippy", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	for _, project := range allProjects {
		cargoArgs := []string{
			"clippy",
			"--manifest-path", filepath.Join(repoRoot, project.CargoToml),
			"--all-targets",
			"--",
			"-D", "warnings",
		}

		if err := runCommand(repoRoot, "cargo", cargoArgs, nil); err != nil {
			return exitExecution
		}
	}

	return exitSuccess
}

func runTest(args []string) int {
	fs := flag.NewFlagSet("test", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	release := fs.Bool("release", false, "Run tests in release mode")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	for _, project := range allProjects {
		cargoArgs := []string{"test"}
		if *release {
			cargoArgs = append(cargoArgs, "--release")
		}
		cargoArgs = append(cargoArgs, "--manifest-path", filepath.Join(repoRoot, project.CargoToml))

		if err := runCommand(repoRoot, "cargo", cargoArgs, nil); err != nil {
			return exitExecution
		}
	}

	return exitSuccess
}

func runBuild(args []string) int {
	fs := flag.NewFlagSet("build", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	release := fs.Bool("release", false, "Build in release mode")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	for _, project := range allProjects {
		cargoArgs := []string{"build"}
		if *release {
			cargoArgs = append(cargoArgs, "--release")
		}
		cargoArgs = append(cargoArgs, "--manifest-path", filepath.Join(repoRoot, project.CargoToml))

		if err := runCommand(repoRoot, "cargo", cargoArgs, nil); err != nil {
			return exitExecution
		}
	}

	return exitSuccess
}

func runDoc(args []string) int {
	fs := flag.NewFlagSet("doc", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	for _, project := range allProjects {
		cargoArgs := []string{
			"doc",
			"--no-deps",
			"--manifest-path", filepath.Join(repoRoot, project.CargoToml),
		}
		env := map[string]string{"RUSTDOCFLAGS": "-D warnings"}
		if err := runCommand(repoRoot, "cargo", cargoArgs, env); err != nil {
			return exitExecution
		}
	}

	return exitSuccess
}

type stringSliceFlag []string

func (s *stringSliceFlag) String() string {
	return strings.Join(*s, ",")
}

func (s *stringSliceFlag) Set(value string) error {
	*s = append(*s, value)
	return nil
}

func runAudit(args []string) int {
	fs := flag.NewFlagSet("audit", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	var ignoreIDs stringSliceFlag
	for _, id := range defaultAuditIgnores {
		ignoreIDs = append(ignoreIDs, id)
	}
	fs.Var(&ignoreIDs, "ignore", "Add advisory ID to ignore list (repeatable)")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	denyConfig := filepath.Join(repoRoot, "deny.toml")
	auditDB := filepath.Join(repoRoot, ".cargo-advisory-db")

	for _, project := range allProjects {
		auditArgs := []string{"audit", "-d", auditDB, "-f", filepath.Join(repoRoot, project.CargoLock)}
		for _, id := range ignoreIDs {
			auditArgs = append(auditArgs, "--ignore", id)
		}

		if err := runCommand(repoRoot, "cargo", auditArgs, nil); err != nil {
			return exitExecution
		}

		denyArgs := []string{"deny", "check", "advisories", "bans", "sources", "--config", denyConfig}
		projectDir := filepath.Join(repoRoot, project.DirRelPath)
		if err := runCommand(projectDir, "cargo", denyArgs, nil); err != nil {
			return exitExecution
		}
	}

	return exitSuccess
}

func runCheck(args []string) int {
	fs := flag.NewFlagSet("check", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if code := runFmt([]string{"--check"}); code != exitSuccess {
		return code
	}
	if code := runClippy(nil); code != exitSuccess {
		return code
	}
	if code := runTest(nil); code != exitSuccess {
		return code
	}

	return exitSuccess
}

func runPreflight(args []string) int {
	fs := flag.NewFlagSet("preflight", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if code := runWorkspaceCleanup(nil); code != exitSuccess {
		return code
	}
	if code := runWorkspaceGuard(nil); code != exitSuccess {
		return code
	}
	if code := runCheck(nil); code != exitSuccess {
		return code
	}
	if code := runTest([]string{"--release"}); code != exitSuccess {
		return code
	}
	if code := runBuild([]string{"--release"}); code != exitSuccess {
		return code
	}
	if code := runDoc(nil); code != exitSuccess {
		return code
	}

	return exitSuccess
}

func runGitCheck(args []string) int {
	fs := flag.NewFlagSet("git-check", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	prePush := fs.Bool("pre-push", false, "Enable pre-push checks (including remote sync and protected-branch block)")
	commitMsgFile := fs.String("commit-msg-file", "", "Commit message file path for commit-msg hook validation")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	if strings.TrimSpace(*commitMsgFile) != "" {
		if err := validateCommitMessageFile(repoRoot, *commitMsgFile); err != nil {
			fmt.Fprintf(os.Stderr, "commit message validation failed: %v\n", err)
			return exitExecution
		}

		fmt.Println("git-check passed (commit message mode)")
		return exitSuccess
	}

	failed := false
	if err := checkBranchProtection(repoRoot, *prePush); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		failed = true
	}

	if err := checkStagedFiles(repoRoot); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		failed = true
	}

	if *prePush {
		if err := checkRemoteSync(repoRoot); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			failed = true
		}
	}

	if failed {
		fmt.Fprintln(os.Stderr, "git-check failed")
		return exitExecution
	}

	fmt.Println("git-check passed")
	return exitSuccess
}

func runRustReview(args []string) int {
	fs := flag.NewFlagSet("rust-review", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	fix := fs.Bool("fix", false, "Auto-format Rust code before checks")
	skipTests := fs.Bool("skip-tests", false, "Skip cargo test")
	skipAudit := fs.Bool("skip-audit", false, "Skip cargo-audit and cargo-deny")
	var projectNames stringSliceFlag
	fs.Var(&projectNames, "project", "Project name to review (repeatable)")
	fs.Var(&projectNames, "projects", "Project name to review (repeatable)")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	selectedProjects, err := resolveSelectedProjects(projectNames)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitUsage
	}

	denyConfig := filepath.Join(repoRoot, "deny.toml")
	auditDB := filepath.Join(repoRoot, ".cargo-advisory-db")

	for _, project := range selectedProjects {
		fmt.Printf("[review] project: %s\n", project.Name)
		manifestPath := filepath.Join(repoRoot, project.CargoToml)

		fmtArgs := []string{"fmt"}
		if !*fix {
			fmtArgs = append(fmtArgs, "--check")
		}
		fmtArgs = append(fmtArgs, "--manifest-path", manifestPath)
		if err := runCommand(repoRoot, "cargo", fmtArgs, nil); err != nil {
			return exitExecution
		}

		clippyArgs := []string{
			"clippy",
			"--manifest-path", manifestPath,
			"--all-targets",
			"--",
			"-D", "warnings",
		}
		if err := runCommand(repoRoot, "cargo", clippyArgs, nil); err != nil {
			return exitExecution
		}

		if !*skipTests {
			testArgs := []string{"test", "--manifest-path", manifestPath}
			if err := runCommand(repoRoot, "cargo", testArgs, nil); err != nil {
				return exitExecution
			}
		}

		docArgs := []string{"doc", "--no-deps", "--manifest-path", manifestPath}
		if err := runCommand(repoRoot, "cargo", docArgs, map[string]string{"RUSTDOCFLAGS": "-D warnings"}); err != nil {
			return exitExecution
		}

		if !*skipAudit {
			auditArgs := []string{"audit", "-d", auditDB, "-f", filepath.Join(repoRoot, project.CargoLock)}
			for _, id := range defaultAuditIgnores {
				auditArgs = append(auditArgs, "--ignore", id)
			}
			if err := runCommand(repoRoot, "cargo", auditArgs, nil); err != nil {
				return exitExecution
			}

			projectDir := filepath.Join(repoRoot, project.DirRelPath)
			denyArgs := []string{"deny", "check", "advisories", "bans", "sources", "--config", denyConfig}
			if err := runCommand(projectDir, "cargo", denyArgs, nil); err != nil {
				return exitExecution
			}
		}
	}

	fmt.Println("rust-review passed")
	return exitSuccess
}

func runReview(args []string) int {
	fs := flag.NewFlagSet("review", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	quick := fs.Bool("quick", false, "Quick mode: only formatting and clippy")
	fix := fs.Bool("fix", false, "Auto-format code in review")
	beforePush := fs.Bool("before-push", false, "Before-push mode: include git pre-push policy and preflight")
	skipTests := fs.Bool("skip-tests", false, "Skip tests in rust-review")
	skipAudit := fs.Bool("skip-audit", false, "Skip audit in rust-review")
	var projectNames stringSliceFlag
	fs.Var(&projectNames, "project", "Project name to review (repeatable)")
	fs.Var(&projectNames, "projects", "Project name to review (repeatable)")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	selectedProjects, err := resolveSelectedProjects(projectNames)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitUsage
	}

	if *quick {
		if *beforePush {
			fmt.Fprintln(os.Stderr, "warning: --before-push is ignored in --quick mode")
		}

		if err := runQuickReview(repoRoot, selectedProjects, *fix); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		fmt.Println("review passed (quick)")
		return exitSuccess
	}

	gitCheckArgs := make([]string, 0)
	if *beforePush {
		gitCheckArgs = append(gitCheckArgs, "--pre-push")
	}
	if code := runGitCheck(gitCheckArgs); code != exitSuccess {
		return code
	}

	rustReviewArgs := make([]string, 0)
	if *fix {
		rustReviewArgs = append(rustReviewArgs, "--fix")
	}
	if *skipTests {
		rustReviewArgs = append(rustReviewArgs, "--skip-tests")
	}
	if *skipAudit {
		rustReviewArgs = append(rustReviewArgs, "--skip-audit")
	}
	for _, name := range projectNames {
		rustReviewArgs = append(rustReviewArgs, "--project", name)
	}

	if code := runRustReview(rustReviewArgs); code != exitSuccess {
		return code
	}

	if *beforePush {
		if code := runPreflight(nil); code != exitSuccess {
			return code
		}
	}

	fmt.Println("review passed")
	return exitSuccess
}

func runInstallHooks(args []string) int {
	fs := flag.NewFlagSet("install-hooks", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	uninstall := fs.Bool("uninstall", false, "Uninstall managed hooks and restore backups")
	force := fs.Bool("force", false, "Force backup/overwrite existing hooks")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	gitDir := filepath.Join(repoRoot, ".git")
	if !fileExists(gitDir) {
		fmt.Fprintln(os.Stderr, "current repository has no .git directory")
		return exitPrecondition
	}

	hooksDir := filepath.Join(gitDir, "hooks")
	if err := os.MkdirAll(hooksDir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create hooks dir: %v\n", err)
		return exitExecution
	}

	hookNames := []string{"pre-commit", "pre-push", "commit-msg"}

	if *uninstall {
		for _, hookName := range hookNames {
			target := filepath.Join(hooksDir, hookName)
			backup := target + ".backup"

			if fileExists(target) {
				content, readErr := os.ReadFile(target)
				if readErr != nil {
					fmt.Fprintf(os.Stderr, "failed to read hook %s: %v\n", hookName, readErr)
					return exitExecution
				}

				if strings.Contains(string(content), "managed-by-rusktask") || *force {
					if err := os.Remove(target); err != nil {
						fmt.Fprintf(os.Stderr, "failed to remove hook %s: %v\n", hookName, err)
						return exitExecution
					}
					fmt.Printf("Removed managed hook: %s\n", hookName)
				}
			}

			if fileExists(backup) {
				if fileExists(target) {
					if err := os.Remove(target); err != nil {
						fmt.Fprintf(os.Stderr, "failed to replace hook %s with backup: %v\n", hookName, err)
						return exitExecution
					}
				}
				if err := os.Rename(backup, target); err != nil {
					fmt.Fprintf(os.Stderr, "failed to restore backup for %s: %v\n", hookName, err)
					return exitExecution
				}
				fmt.Printf("Restored backup hook: %s\n", hookName)
			}
		}

		fmt.Println("install-hooks uninstall completed")
		return exitSuccess
	}

	for _, hookName := range hookNames {
		target := filepath.Join(hooksDir, hookName)
		backup := target + ".backup"

		if fileExists(target) {
			content, readErr := os.ReadFile(target)
			if readErr != nil {
				fmt.Fprintf(os.Stderr, "failed to read existing hook %s: %v\n", hookName, readErr)
				return exitExecution
			}

			managed := strings.Contains(string(content), "managed-by-rusktask")
			if !managed || *force {
				if fileExists(backup) {
					if err := os.Remove(backup); err != nil {
						fmt.Fprintf(os.Stderr, "failed to remove old backup for %s: %v\n", hookName, err)
						return exitExecution
					}
				}
				if err := os.Rename(target, backup); err != nil {
					fmt.Fprintf(os.Stderr, "failed to backup existing hook %s: %v\n", hookName, err)
					return exitExecution
				}
				fmt.Printf("Backed up existing hook: %s -> %s\n", hookName, filepath.Base(backup))
			}
		}

		script, err := buildManagedHookScript(hookName)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		if err := writeTextFile(target, script, 0o755); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}
		if err := os.Chmod(target, 0o755); err != nil {
			fmt.Fprintf(os.Stderr, "failed to mark hook executable %s: %v\n", hookName, err)
			return exitExecution
		}

		fmt.Printf("Installed hook: %s\n", hookName)
	}

	fmt.Println("install-hooks completed")
	return exitSuccess
}

func runSmartBump(args []string) int {
	fs := flag.NewFlagSet("smart-bump", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	part := fs.String("part", "patch", "Version bump part: patch, minor, major")
	push := fs.Bool("push", false, "Push branch and tag to origin after bump")
	noVerify := fs.Bool("no-verify", false, "Use --no-verify when pushing")
	allowDirty := fs.Bool("allow-dirty", false, "Allow dirty working tree")
	noTag := fs.Bool("no-tag", false, "Skip annotated tag creation")
	skipReleaseStateAudit := fs.Bool("skip-release-state-audit", false, "Skip strict release-state audit before bump")
	skipProcessCleanup := fs.Bool("skip-process-cleanup", false, "Skip workspace cleanup before and after bump")
	skipWorkspaceGuard := fs.Bool("skip-workspace-guard", false, "Skip workspace structure guard before and after bump")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if *part != "patch" && *part != "minor" && *part != "major" {
		fmt.Fprintln(os.Stderr, "--part must be one of: patch, minor, major")
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	if code := runSmartMaintenance(*skipProcessCleanup, *skipWorkspaceGuard); code != exitSuccess {
		return code
	}

	mainCode := func() int {
		if !*allowDirty {
			if err := ensureCleanWorktree(repoRoot); err != nil {
				fmt.Fprintf(os.Stderr, "%v\n", err)
				return exitExecution
			}
		}

		if err := ensureReleaseBranch(repoRoot); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		if !*skipReleaseStateAudit {
			if code := runReleaseSync([]string{"--mode", "audit", "--strict"}); code != exitSuccess {
				return code
			}
		}

		manifestRelPaths := []string{
			filepath.ToSlash("robot_control_rust/Cargo.toml"),
			filepath.ToSlash("rust_tools_suite/Cargo.toml"),
		}

		anchorManifest := filepath.Join(repoRoot, filepath.FromSlash("robot_control_rust/Cargo.toml"))
		currentVersion, err := parseManifestVersion(anchorManifest)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		nextVersion, err := nextSemverVersion(currentVersion, *part)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		tagName := "v" + nextVersion
		if err := ensureTagNotExists(repoRoot, tagName); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		updatedManifests := make([]string, 0, len(manifestRelPaths))
		for _, relPath := range manifestRelPaths {
			absPath := filepath.Join(repoRoot, filepath.FromSlash(relPath))
			if !fileExists(absPath) {
				continue
			}

			if err := updateManifestVersion(absPath, nextVersion); err != nil {
				fmt.Fprintf(os.Stderr, "%v\n", err)
				return exitExecution
			}

			updatedManifests = append(updatedManifests, relPath)
			fmt.Printf("Updated %s -> %s\n", relPath, nextVersion)
		}

		if len(updatedManifests) == 0 {
			fmt.Fprintln(os.Stderr, "no manifest files updated for version bump")
			return exitPrecondition
		}

		releaseNotesRelPath := fmt.Sprintf("release_notes/RELEASE_NOTES_%s.md", tagName)
		releaseNotesAbsPath := filepath.Join(repoRoot, filepath.FromSlash(releaseNotesRelPath))
		if err := createReleaseNotesDraft(releaseNotesAbsPath, tagName); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		if code := runReleaseNotesValidate([]string{"--file", releaseNotesAbsPath, "--mode", "draft"}); code != exitSuccess {
			return code
		}

		if err := updateReleaseIndex(repoRoot, false); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		releaseIndexRelPath := filepath.ToSlash("release_notes/RELEASE_INDEX.md")
		addArgs := append([]string{"add"}, updatedManifests...)
		addArgs = append(addArgs, releaseNotesRelPath, releaseIndexRelPath)
		if err := runCommand(repoRoot, "git", addArgs, nil); err != nil {
			return exitExecution
		}

		commitMessage := fmt.Sprintf("chore(release): bump version to %s", tagName)
		if err := runCommand(repoRoot, "git", []string{"commit", "-m", commitMessage}, nil); err != nil {
			return exitExecution
		}

		if !*noTag {
			if err := runCommand(repoRoot, "git", []string{"tag", "-a", tagName, "-m", "Release " + tagName}, nil); err != nil {
				return exitExecution
			}
			fmt.Printf("Created tag: %s\n", tagName)
		}

		if *push {
			pushBranchArgs := []string{"push"}
			if *noVerify {
				pushBranchArgs = append(pushBranchArgs, "--no-verify")
			}
			pushBranchArgs = append(pushBranchArgs, "origin", "HEAD")
			if err := runCommand(repoRoot, "git", pushBranchArgs, nil); err != nil {
				return exitExecution
			}

			if !*noTag {
				pushTagArgs := []string{"push"}
				if *noVerify {
					pushTagArgs = append(pushTagArgs, "--no-verify")
				}
				pushTagArgs = append(pushTagArgs, "origin", tagName)
				if err := runCommand(repoRoot, "git", pushTagArgs, nil); err != nil {
					return exitExecution
				}
			}

			fmt.Println("Pushed branch and tag to origin")
		}

		fmt.Printf("Release bump completed: %s -> %s\n", currentVersion, nextVersion)
		return exitSuccess
	}()

	postCode := runSmartMaintenance(*skipProcessCleanup, *skipWorkspaceGuard)
	if mainCode != exitSuccess {
		if postCode != exitSuccess {
			fmt.Fprintln(os.Stderr, "warning: post-bump maintenance also failed")
		}
		return mainCode
	}

	if postCode != exitSuccess {
		return postCode
	}

	return exitSuccess
}

func runSmartRollback(args []string) int {
	fs := flag.NewFlagSet("smart-rollback", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	tag := fs.String("tag", "", "Tag to rollback, e.g. v0.1.8")
	owner := fs.String("owner", "loopgap", "GitHub owner")
	repo := fs.String("repo", "robot_ctrl_rust_app", "GitHub repository")
	deleteRelease := fs.Bool("delete-release", false, "Delete GitHub release for the given tag")
	deleteRemoteTag := fs.Bool("delete-remote-tag", false, "Delete remote git tag")
	deleteLocalTag := fs.Bool("delete-local-tag", false, "Delete local git tag")
	revertLastCommit := fs.Bool("revert-last-commit", false, "Revert latest release bump commit")
	pushRevert := fs.Bool("push-revert", false, "Push revert commit after reverting")
	noVerify := fs.Bool("no-verify", false, "Use --no-verify when pushing")
	skipProcessCleanup := fs.Bool("skip-process-cleanup", false, "Skip workspace cleanup before and after rollback")
	skipWorkspaceGuard := fs.Bool("skip-workspace-guard", false, "Skip workspace structure guard before and after rollback")
	skipIndexRefresh := fs.Bool("skip-index-refresh", false, "Skip release index refresh")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	trimmedTag := strings.TrimSpace(*tag)
	if !regexp.MustCompile(`^v\d+\.\d+\.\d+([-.].+)?$`).MatchString(trimmedTag) {
		fmt.Fprintln(os.Stderr, "--tag is required and must match vX.Y.Z or vX.Y.Z-suffix")
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	if code := runSmartMaintenance(*skipProcessCleanup, *skipWorkspaceGuard); code != exitSuccess {
		return code
	}

	mainCode := func() int {
		if *deleteRelease {
			token := strings.TrimSpace(os.Getenv("GITHUB_TOKEN"))
			if token == "" {
				fmt.Fprintln(os.Stderr, "DeleteRelease requires GITHUB_TOKEN")
				return exitPrecondition
			}

			if err := deleteGitHubReleaseByTag(strings.TrimSpace(*owner), strings.TrimSpace(*repo), trimmedTag, token); err != nil {
				fmt.Fprintf(os.Stderr, "%v\n", err)
				return exitExecution
			}
		}

		if *deleteRemoteTag {
			pushArgs := []string{"push"}
			if *noVerify {
				pushArgs = append(pushArgs, "--no-verify")
			}
			pushArgs = append(pushArgs, "origin", ":refs/tags/"+trimmedTag)
			if err := runCommand(repoRoot, "git", pushArgs, nil); err != nil {
				return exitExecution
			}
			fmt.Printf("Deleted remote tag: %s\n", trimmedTag)
		}

		if *deleteLocalTag {
			if err := runCommand(repoRoot, "git", []string{"tag", "-d", trimmedTag}, nil); err != nil {
				return exitExecution
			}
			fmt.Printf("Deleted local tag: %s\n", trimmedTag)
		}

		if *revertLastCommit {
			latestCommitMessage, err := runCommandCapture(repoRoot, "git", []string{"log", "-1", "--pretty=%s"})
			if err != nil {
				fmt.Fprintln(os.Stderr, "Failed to inspect latest commit")
				return exitExecution
			}

			if !strings.HasPrefix(strings.TrimSpace(latestCommitMessage), "chore(release): bump version to ") {
				fmt.Fprintf(os.Stderr, "Latest commit is not a release bump commit: %s\n", strings.TrimSpace(latestCommitMessage))
				return exitExecution
			}

			if err := runCommand(repoRoot, "git", []string{"revert", "--no-edit", "HEAD"}, nil); err != nil {
				return exitExecution
			}
			fmt.Println("Reverted last release bump commit")

			if *pushRevert {
				pushArgs := []string{"push"}
				if *noVerify {
					pushArgs = append(pushArgs, "--no-verify")
				}
				pushArgs = append(pushArgs, "origin", "HEAD")
				if err := runCommand(repoRoot, "git", pushArgs, nil); err != nil {
					return exitExecution
				}
				fmt.Println("Pushed revert commit to origin")
			}
		}

		if !*skipIndexRefresh {
			if err := updateReleaseIndex(repoRoot, false); err != nil {
				fmt.Fprintf(os.Stderr, "%v\n", err)
				return exitExecution
			}
		}

		fmt.Println("Rollback operation completed.")
		return exitSuccess
	}()

	postCode := runSmartMaintenance(*skipProcessCleanup, *skipWorkspaceGuard)
	if mainCode != exitSuccess {
		if postCode != exitSuccess {
			fmt.Fprintln(os.Stderr, "warning: post-rollback maintenance also failed")
		}
		return mainCode
	}

	if postCode != exitSuccess {
		return postCode
	}

	return exitSuccess
}

func runPRHelper(args []string) int {
	fs := flag.NewFlagSet("pr-helper", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	create := fs.Bool("create", false, "Create a new pull request")
	check := fs.Bool("check", false, "Check pull request readiness")
	merge := fs.Bool("merge", false, "Merge current pull request")
	draft := fs.Bool("draft", false, "Create draft pull request")
	title := fs.String("title", "", "Pull request title")
	body := fs.String("body", "", "Pull request body")
	base := fs.String("base", "main", "Base branch")
	head := fs.String("head", "", "Head branch")
	autoFill := fs.Bool("auto-fill", false, "Auto generate pull request body")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	modeCount := 0
	for _, enabled := range []bool{*create, *check, *merge} {
		if enabled {
			modeCount++
		}
	}
	if modeCount > 1 {
		fmt.Fprintln(os.Stderr, "only one of --create/--check/--merge can be used at a time")
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	currentBranchOut, err := runCommandCapture(repoRoot, "git", []string{"rev-parse", "--abbrev-ref", "HEAD"})
	if err != nil {
		fmt.Fprintln(os.Stderr, "无法获取当前分支")
		return exitExecution
	}

	headBranch := strings.TrimSpace(*head)
	if headBranch == "" {
		headBranch = strings.TrimSpace(currentBranchOut)
	}
	baseBranch := strings.TrimSpace(*base)
	if baseBranch == "" {
		baseBranch = "main"
	}

	fmt.Printf("当前分支: %s\n", headBranch)
	fmt.Printf("目标分支: %s\n", baseBranch)

	if *merge {
		ghPath, lookErr := exec.LookPath("gh")
		if lookErr != nil {
			fmt.Fprintln(os.Stderr, "未找到 GitHub CLI(gh)，请手动合并 PR")
			return exitPrecondition
		}

		if _, err := runCommandCapture(repoRoot, ghPath, []string{"pr", "view", "--json", "state,mergeStateStatus,title"}); err != nil {
			fmt.Fprintln(os.Stderr, "当前分支没有关联的 PR")
			return exitExecution
		}

		if err := runCommand(repoRoot, ghPath, []string{"pr", "merge", "--squash", "--delete-branch"}, nil); err != nil {
			return exitExecution
		}

		fmt.Println("PR 合并成功")
		return exitSuccess
	}

	issues, warnings := evaluatePRReadiness(repoRoot, baseBranch, headBranch)
	if len(issues) == 0 && len(warnings) == 0 {
		fmt.Println("PR 准备就绪")
	} else {
		if len(issues) > 0 {
			fmt.Println("发现以下问题，需要修复:")
			for _, issue := range issues {
				fmt.Printf("  - %s\n", issue)
			}
		}
		if len(warnings) > 0 {
			fmt.Println("警告:")
			for _, warning := range warnings {
				fmt.Printf("  - %s\n", warning)
			}
		}
	}

	if *check || (!*create && !*merge) {
		if len(issues) == 0 && len(warnings) == 0 {
			return exitSuccess
		}
		return exitExecution
	}

	if len(issues) > 0 || len(warnings) > 0 {
		fmt.Fprintln(os.Stderr, "PR 准备检查未通过，无法创建 PR")
		return exitExecution
	}

	resolvedTitle := strings.TrimSpace(*title)
	if resolvedTitle == "" {
		out, err := runCommandCapture(repoRoot, "git", []string{"log", "--pretty=format:%s", "-1", fmt.Sprintf("origin/%s..%s", baseBranch, headBranch)})
		if err != nil {
			fmt.Fprintln(os.Stderr, "无法自动生成 PR 标题，请使用 --title")
			return exitExecution
		}
		resolvedTitle = strings.TrimSpace(out)
		if resolvedTitle == "" {
			resolvedTitle = fmt.Sprintf("Merge %s into %s", headBranch, baseBranch)
		}
	}

	resolvedBody := strings.TrimSpace(*body)
	if resolvedBody == "" && *autoFill {
		autoBody, err := generateAutoPRDescription(repoRoot, baseBranch, headBranch)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}
		resolvedBody = autoBody
	}

	ghPath, lookErr := exec.LookPath("gh")
	if lookErr != nil {
		fmt.Fprintln(os.Stderr, "未找到 GitHub CLI(gh)，请手动创建 PR")
		fmt.Printf("建议标题: %s\n", resolvedTitle)
		if resolvedBody == "" {
			autoBody, err := generateAutoPRDescription(repoRoot, baseBranch, headBranch)
			if err == nil {
				resolvedBody = autoBody
			}
		}
		if resolvedBody != "" {
			fmt.Println("建议描述:")
			fmt.Println(resolvedBody)
		}
		return exitPrecondition
	}

	ghArgs := []string{"pr", "create", "--base", baseBranch, "--head", headBranch, "--title", resolvedTitle}
	if *draft {
		ghArgs = append(ghArgs, "--draft")
	}

	var tempBodyFile string
	if resolvedBody != "" {
		tmpFile, err := os.CreateTemp("", "rusktask-pr-body-*.md")
		if err != nil {
			fmt.Fprintf(os.Stderr, "failed to create temp body file: %v\n", err)
			return exitExecution
		}

		tempBodyFile = tmpFile.Name()
		if _, err := tmpFile.WriteString(resolvedBody); err != nil {
			tmpFile.Close()
			_ = os.Remove(tempBodyFile)
			fmt.Fprintf(os.Stderr, "failed to write temp body file: %v\n", err)
			return exitExecution
		}
		if err := tmpFile.Close(); err != nil {
			_ = os.Remove(tempBodyFile)
			fmt.Fprintf(os.Stderr, "failed to close temp body file: %v\n", err)
			return exitExecution
		}

		defer func() {
			_ = os.Remove(tempBodyFile)
		}()

		ghArgs = append(ghArgs, "--body-file", tempBodyFile)
	}

	if err := runCommand(repoRoot, ghPath, ghArgs, nil); err != nil {
		return exitExecution
	}

	fmt.Println("PR 创建成功")
	return exitSuccess
}

func runSmartMaintenance(skipProcessCleanup bool, skipWorkspaceGuard bool) int {
	if !skipProcessCleanup {
		if code := runWorkspaceCleanup([]string{"--mode", "apply"}); code != exitSuccess {
			return code
		}
	}

	if !skipWorkspaceGuard {
		if code := runWorkspaceGuard([]string{"--mode", "audit", "--strict"}); code != exitSuccess {
			return code
		}
	}

	return exitSuccess
}

func ensureCleanWorktree(repoRoot string) error {
	statusOutput, err := runCommandCapture(repoRoot, "git", []string{"status", "--porcelain"})
	if err != nil {
		return errors.New("failed to inspect working tree status")
	}

	if strings.TrimSpace(statusOutput) != "" {
		return errors.New("working tree is not clean. Commit or stash changes first, or use --allow-dirty")
	}

	return nil
}

func ensureReleaseBranch(repoRoot string) error {
	branchOutput, err := runCommandCapture(repoRoot, "git", []string{"rev-parse", "--abbrev-ref", "HEAD"})
	if err != nil {
		return errors.New("failed to resolve current branch")
	}

	branch := strings.TrimSpace(branchOutput)
	if branch != "main" && branch != "master" {
		return fmt.Errorf("release bump must run on main/master. Current branch: %s", branch)
	}

	return nil
}

func parseManifestVersion(manifestPath string) (string, error) {
	content, err := os.ReadFile(manifestPath)
	if err != nil {
		return "", fmt.Errorf("failed to read manifest %s: %w", manifestPath, err)
	}

	match := regexp.MustCompile(`(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"\s*$`).FindStringSubmatch(string(content))
	if len(match) != 2 {
		return "", fmt.Errorf("cannot read semantic version from %s", manifestPath)
	}

	return strings.TrimSpace(match[1]), nil
}

func nextSemverVersion(current string, part string) (string, error) {
	segments := strings.Split(strings.TrimSpace(current), ".")
	if len(segments) != 3 {
		return "", fmt.Errorf("invalid semantic version: %s", current)
	}

	major, err := strconv.Atoi(segments[0])
	if err != nil {
		return "", fmt.Errorf("invalid major version segment: %s", segments[0])
	}
	minor, err := strconv.Atoi(segments[1])
	if err != nil {
		return "", fmt.Errorf("invalid minor version segment: %s", segments[1])
	}
	patch, err := strconv.Atoi(segments[2])
	if err != nil {
		return "", fmt.Errorf("invalid patch version segment: %s", segments[2])
	}

	switch part {
	case "major":
		major++
		minor = 0
		patch = 0
	case "minor":
		minor++
		patch = 0
	default:
		patch++
	}

	return fmt.Sprintf("%d.%d.%d", major, minor, patch), nil
}

func updateManifestVersion(manifestPath string, newVersion string) error {
	content, err := os.ReadFile(manifestPath)
	if err != nil {
		return fmt.Errorf("failed to read manifest %s: %w", manifestPath, err)
	}

	rx := regexp.MustCompile(`(?m)^version\s*=\s*"\d+\.\d+\.\d+"\s*$`)
	loc := rx.FindIndex(content)
	if loc == nil {
		return fmt.Errorf("manifest %s does not contain a semantic version line", manifestPath)
	}

	replacement := []byte(fmt.Sprintf("version = \"%s\"", newVersion))
	updated := make([]byte, 0, len(content)-loc[1]+loc[0]+len(replacement))
	updated = append(updated, content[:loc[0]]...)
	updated = append(updated, replacement...)
	updated = append(updated, content[loc[1]:]...)

	info, err := os.Stat(manifestPath)
	if err != nil {
		return fmt.Errorf("failed to stat manifest %s: %w", manifestPath, err)
	}

	if err := os.WriteFile(manifestPath, updated, info.Mode()); err != nil {
		return fmt.Errorf("failed to update manifest %s: %w", manifestPath, err)
	}

	return nil
}

func ensureTagNotExists(repoRoot string, tagName string) error {
	localTags, err := runCommandCapture(repoRoot, "git", []string{"tag", "-l", tagName})
	if err != nil {
		return errors.New("failed to inspect local tags")
	}
	if strings.TrimSpace(localTags) != "" {
		return fmt.Errorf("tag already exists locally: %s", tagName)
	}

	if err := runCommand(repoRoot, "git", []string{"fetch", "--tags", "--quiet"}, nil); err != nil {
		return fmt.Errorf("failed to fetch remote tags: %w", err)
	}

	remoteTags, err := runCommandCapture(repoRoot, "git", []string{"ls-remote", "--tags", "origin", tagName})
	if err != nil {
		return fmt.Errorf("failed to inspect remote tags: %w", err)
	}
	if strings.TrimSpace(remoteTags) != "" {
		return fmt.Errorf("tag already exists on origin: %s", tagName)
	}

	return nil
}

func createReleaseNotesDraft(releaseNotesPath string, tagName string) error {
	if err := os.MkdirAll(filepath.Dir(releaseNotesPath), 0o755); err != nil {
		return fmt.Errorf("failed to create release notes directory: %w", err)
	}

	notes := fmt.Sprintf(`# %s

## Highlights
- Describe major improvements here.

## Fixes
- Describe bug fixes here.

## Verification
- [ ] scripts/task preflight
- [ ] CI passed
- [ ] Release assets verified (exe/setup/checksums)
`, tagName)

	if err := os.WriteFile(releaseNotesPath, []byte(notes), 0o644); err != nil {
		return fmt.Errorf("failed to write release notes draft %s: %w", releaseNotesPath, err)
	}

	return nil
}

func deleteGitHubReleaseByTag(owner string, repo string, tag string, token string) error {
	if strings.TrimSpace(owner) == "" || strings.TrimSpace(repo) == "" {
		return errors.New("owner and repo must not be empty")
	}

	getURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases/tags/%s", owner, repo, url.PathEscape(tag))
	body, statusCode, err := doGitHubRequest("GET", getURL, token, nil, "")
	if err != nil {
		if statusCode == http.StatusNotFound {
			fmt.Printf("Release for tag %s not found. Skip delete.\n", tag)
			return nil
		}
		return err
	}

	release := githubRelease{}
	if err := json.Unmarshal(body, &release); err != nil {
		return fmt.Errorf("failed to parse release info for tag %s: %w", tag, err)
	}

	deleteURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases/%d", owner, repo, release.ID)
	if _, _, err := doGitHubRequest("DELETE", deleteURL, token, nil, ""); err != nil {
		return err
	}

	fmt.Printf("Deleted GitHub release for %s\n", tag)
	return nil
}

func evaluatePRReadiness(repoRoot string, baseBranch string, headBranch string) ([]string, []string) {
	issues := make([]string, 0)
	warnings := make([]string, 0)

	if headBranch == baseBranch {
		issues = append(issues, fmt.Sprintf("不能在目标分支 '%s' 上创建 PR", baseBranch))
	}

	remoteBranchOutput, err := runCommandCapture(repoRoot, "git", []string{"ls-remote", "--heads", "origin", headBranch})
	if err != nil || strings.TrimSpace(remoteBranchOutput) == "" {
		issues = append(issues, fmt.Sprintf("分支 '%s' 未推送到远程，请先执行 git push -u origin %s", headBranch, headBranch))
	}

	statusOutput, err := runCommandCapture(repoRoot, "git", []string{"status", "--porcelain"})
	if err != nil {
		issues = append(issues, "无法检查工作区状态")
	} else if strings.TrimSpace(statusOutput) != "" {
		warnings = append(warnings, "工作区有未提交的更改")
	}

	if err := runCommand(repoRoot, "git", []string{"fetch", "origin", baseBranch, "--quiet"}, nil); err != nil {
		issues = append(issues, fmt.Sprintf("无法获取 origin/%s: %v", baseBranch, err))
	}

	rangeExpr := fmt.Sprintf("origin/%s...%s", baseBranch, headBranch)
	diffOutput, err := runCommandCapture(repoRoot, "git", []string{"diff", "--stat", rangeExpr})
	if err != nil {
		issues = append(issues, "无法比较分支差异")
	} else if strings.TrimSpace(diffOutput) == "" {
		issues = append(issues, "与目标分支没有差异，无需创建 PR")
	} else {
		commitCountOutput, countErr := runCommandCapture(repoRoot, "git", []string{"rev-list", "--count", rangeExpr})
		if countErr == nil {
			fmt.Printf("提交数: %s\n", strings.TrimSpace(commitCountOutput))
		}

		fileCount := 0
		for _, line := range strings.Split(strings.ReplaceAll(diffOutput, "\r\n", "\n"), "\n") {
			if strings.TrimSpace(line) == "" {
				continue
			}
			if strings.Contains(line, "|") {
				fileCount++
			}
		}
		fmt.Printf("修改文件数: %d\n", fileCount)
	}

	mergeBaseOutput, err := runCommandCapture(repoRoot, "git", []string{"merge-base", "HEAD", fmt.Sprintf("origin/%s", baseBranch)})
	if err != nil {
		issues = append(issues, "无法计算 merge-base")
	} else {
		mergeBase := strings.TrimSpace(mergeBaseOutput)
		mergeTreeOutput, mergeTreeErr := runCommandCapture(repoRoot, "git", []string{"merge-tree", mergeBase, "HEAD", fmt.Sprintf("origin/%s", baseBranch)})
		if mergeTreeErr != nil {
			issues = append(issues, "无法检查合并冲突")
		} else if strings.Contains(mergeTreeOutput, "<<<<<<<") {
			issues = append(issues, "存在合并冲突，请先解决")
		}
	}

	if code := runReview([]string{"--quick"}); code != exitSuccess {
		issues = append(issues, "代码审查未通过")
	}

	commitsOutput, err := runCommandCapture(repoRoot, "git", []string{"log", "--oneline", fmt.Sprintf("origin/%s..%s", baseBranch, headBranch)})
	if err == nil && strings.TrimSpace(commitsOutput) != "" {
		fmt.Println("最近提交:")
		lines := strings.Split(strings.ReplaceAll(commitsOutput, "\r\n", "\n"), "\n")
		for i, line := range lines {
			if strings.TrimSpace(line) == "" {
				continue
			}
			if i >= 5 {
				break
			}
			fmt.Printf("  %s\n", strings.TrimSpace(line))
		}
	}

	return issues, warnings
}

func generateAutoPRDescription(repoRoot string, baseBranch string, headBranch string) (string, error) {
	commitOutput, err := runCommandCapture(repoRoot, "git", []string{"log", "--pretty=format:- %s", fmt.Sprintf("origin/%s..%s", baseBranch, headBranch)})
	if err != nil {
		return "", errors.New("failed to collect commits for PR description")
	}

	filesOutput, err := runCommandCapture(repoRoot, "git", []string{"diff", "--name-only", fmt.Sprintf("origin/%s...%s", baseBranch, headBranch)})
	if err != nil {
		return "", errors.New("failed to collect changed files for PR description")
	}

	files := make([]string, 0)
	for _, line := range strings.Split(strings.ReplaceAll(filesOutput, "\r\n", "\n"), "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}
		files = append(files, trimmed)
	}

	categories := map[string][]string{
		"新功能": {},
		"修复":  {},
		"文档":  {},
		"其他":  {},
	}

	commitLower := strings.ToLower(commitOutput)
	for _, file := range files {
		lowerFile := strings.ToLower(file)
		switch {
		case strings.HasPrefix(lowerFile, "docs/") || strings.HasSuffix(lowerFile, ".md"):
			categories["文档"] = append(categories["文档"], file)
		case strings.Contains(commitLower, "feat") || strings.Contains(commitLower, "add") || strings.Contains(commitLower, "new"):
			if !strings.Contains(lowerFile, "test") {
				categories["新功能"] = append(categories["新功能"], file)
				break
			}
			categories["其他"] = append(categories["其他"], file)
		case strings.Contains(commitLower, "fix") || strings.Contains(commitLower, "bug") || strings.Contains(commitLower, "repair"):
			categories["修复"] = append(categories["修复"], file)
		default:
			categories["其他"] = append(categories["其他"], file)
		}
	}

	orderedCategories := []string{"新功能", "修复", "文档", "其他"}
	b := strings.Builder{}
	b.WriteString("## 变更摘要\n\n")
	b.WriteString("### 提交历史\n")
	b.WriteString(strings.TrimSpace(commitOutput))
	b.WriteString("\n\n### 变更文件\n")
	for _, category := range orderedCategories {
		items := categories[category]
		if len(items) == 0 {
			continue
		}
		b.WriteString("\n#### ")
		b.WriteString(category)
		b.WriteString("\n")
		for _, item := range items {
			b.WriteString("- ")
			b.WriteString(item)
			b.WriteString("\n")
		}
	}

	b.WriteString("\n### 检查清单\n")
	b.WriteString("- [ ] 代码审查通过\n")
	b.WriteString("- [ ] 测试通过\n")
	b.WriteString("- [ ] 文档已更新\n")
	b.WriteString("- [ ] 无合并冲突\n\n")
	b.WriteString("### 相关Issue\n")
	b.WriteString("<!-- 关联的Issue编号，如: Fixes #123 -->\n")

	return b.String(), nil
}

func validateCommitMessageFile(repoRoot string, commitMsgFile string) error {
	path := strings.TrimSpace(commitMsgFile)
	if path == "" {
		return errors.New("commit message file path is empty")
	}

	if !filepath.IsAbs(path) {
		if !fileExists(path) {
			path = filepath.Join(repoRoot, filepath.FromSlash(path))
		}
	}

	content, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("failed to read commit message file %s: %w", path, err)
	}

	firstLine := extractCommitMessageFirstLine(string(content))
	if firstLine == "" {
		return errors.New("commit message is empty")
	}

	if err := validateCommitMessageLine(firstLine); err != nil {
		return err
	}

	return nil
}

func extractCommitMessageFirstLine(message string) string {
	normalized := strings.ReplaceAll(message, "\r\n", "\n")
	for _, line := range strings.Split(normalized, "\n") {
		trimmed := strings.TrimSpace(line)
		trimmed = strings.TrimPrefix(trimmed, "\uFEFF")
		if trimmed == "" {
			continue
		}
		if strings.HasPrefix(trimmed, "#") {
			continue
		}
		return trimmed
	}
	return ""
}

func validateCommitMessageLine(firstLine string) error {
	commitPattern := regexp.MustCompile(`^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?: .{1,100}$`)
	commitDetailPattern := regexp.MustCompile(`^\[.+\] .+`)
	mergePattern := regexp.MustCompile(`^Merge (branch|pull request|remote-tracking branch)`)

	if commitPattern.MatchString(firstLine) {
		return nil
	}
	if commitDetailPattern.MatchString(firstLine) {
		return nil
	}
	if mergePattern.MatchString(firstLine) {
		return nil
	}

	return fmt.Errorf("commit message does not match required format: %s", firstLine)
}

func checkBranchProtection(repoRoot string, prePush bool) error {
	branchOutput, err := runCommandCapture(repoRoot, "git", []string{"rev-parse", "--abbrev-ref", "HEAD"})
	if err != nil {
		return errors.New("failed to get current branch")
	}

	branch := strings.TrimSpace(branchOutput)
	if branch == "" {
		return errors.New("failed to resolve current branch")
	}

	protectedPatterns := []string{"main", "master", "release/*"}
	for _, pattern := range protectedPatterns {
		matched, _ := filepath.Match(pattern, branch)
		if !matched {
			continue
		}

		if prePush {
			return fmt.Errorf("direct push to protected branch is not allowed: %s", branch)
		}

		fmt.Fprintf(os.Stderr, "warning: currently on protected branch: %s\n", branch)
	}

	validPatterns := []string{
		"feature/*",
		"fix/*",
		"docs/*",
		"refactor/*",
		"test/*",
		"chore/*",
		"main",
		"master",
		"develop",
		"release/*",
	}

	isValid := false
	for _, pattern := range validPatterns {
		matched, _ := filepath.Match(pattern, branch)
		if matched {
			isValid = true
			break
		}
	}

	if !isValid {
		fmt.Fprintf(os.Stderr, "warning: branch name does not match recommended patterns: %s\n", branch)
	}

	return nil
}

func checkStagedFiles(repoRoot string) error {
	output, err := runCommandCapture(repoRoot, "git", []string{"diff", "--cached", "--name-only"})
	if err != nil {
		return errors.New("failed to inspect staged files")
	}

	lines := strings.Split(strings.ReplaceAll(output, "\r\n", "\n"), "\n")
	stagedFiles := make([]string, 0, len(lines))
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}
		stagedFiles = append(stagedFiles, trimmed)
	}

	if len(stagedFiles) == 0 {
		fmt.Fprintln(os.Stderr, "warning: staged area is empty")
		return nil
	}

	const maxSize = 1 << 20
	largeFiles := make([]string, 0)
	sensitiveFiles := make([]string, 0)
	sensitivePatterns := []string{"*.pem", "*.key", "*.p12", "*.env", "secrets*", "password*", "credential*"}

	for _, relPath := range stagedFiles {
		size, sizeErr := resolveStagedFileSize(repoRoot, relPath)
		if sizeErr == nil && size > maxSize {
			sizeMB := float64(size) / (1024.0 * 1024.0)
			largeFiles = append(largeFiles, fmt.Sprintf("%s (%.2fMB)", relPath, sizeMB))
		}

		baseName := strings.ToLower(filepath.Base(relPath))
		for _, pattern := range sensitivePatterns {
			matched, _ := filepath.Match(strings.ToLower(pattern), baseName)
			if matched {
				sensitiveFiles = append(sensitiveFiles, relPath)
				break
			}
		}
	}

	if len(largeFiles) == 0 && len(sensitiveFiles) == 0 {
		return nil
	}

	messages := make([]string, 0)
	if len(largeFiles) > 0 {
		messages = append(messages, "large staged files detected: "+strings.Join(largeFiles, ", "))
	}
	if len(sensitiveFiles) > 0 {
		messages = append(messages, "potentially sensitive files detected: "+strings.Join(sensitiveFiles, ", "))
	}

	return errors.New(strings.Join(messages, "; "))
}

func resolveStagedFileSize(repoRoot string, relPath string) (int64, error) {
	sizeOutput, err := runCommandCapture(repoRoot, "git", []string{"cat-file", "-s", ":" + relPath})
	if err == nil {
		parsed, parseErr := strconv.ParseInt(strings.TrimSpace(sizeOutput), 10, 64)
		if parseErr == nil {
			return parsed, nil
		}
	}

	absPath := filepath.Join(repoRoot, filepath.FromSlash(relPath))
	info, statErr := os.Stat(absPath)
	if statErr != nil {
		if err != nil {
			return 0, err
		}
		return 0, statErr
	}

	return info.Size(), nil
}

func checkRemoteSync(repoRoot string) error {
	_, _ = runCommandCapture(repoRoot, "git", []string{"fetch", "origin", "--quiet"})

	branchOutput, err := runCommandCapture(repoRoot, "git", []string{"rev-parse", "--abbrev-ref", "HEAD"})
	if err != nil {
		return errors.New("failed to get current branch for remote sync")
	}
	branch := strings.TrimSpace(branchOutput)
	if branch == "" {
		return errors.New("failed to resolve current branch for remote sync")
	}

	localCommitOut, err := runCommandCapture(repoRoot, "git", []string{"rev-parse", "HEAD"})
	if err != nil {
		return errors.New("failed to resolve local HEAD")
	}
	localCommit := strings.TrimSpace(localCommitOut)

	remoteRef := fmt.Sprintf("origin/%s", branch)
	remoteCommitOut, err := runCommandCapture(repoRoot, "git", []string{"rev-parse", remoteRef})
	if err != nil {
		fmt.Fprintf(os.Stderr, "warning: remote branch not found, skip remote sync check: %s\n", remoteRef)
		return nil
	}
	remoteCommit := strings.TrimSpace(remoteCommitOut)

	baseOut, err := runCommandCapture(repoRoot, "git", []string{"merge-base", "HEAD", remoteRef})
	if err != nil {
		return fmt.Errorf("failed to compute merge-base with %s", remoteRef)
	}
	baseCommit := strings.TrimSpace(baseOut)

	if localCommit == remoteCommit {
		return nil
	}
	if baseCommit == localCommit {
		return fmt.Errorf("local branch is behind %s", remoteRef)
	}
	if baseCommit == remoteCommit {
		return nil
	}

	return fmt.Errorf("local branch has diverged from %s", remoteRef)
}

func resolveSelectedProjects(projectNames []string) ([]projectDef, error) {
	if len(projectNames) == 0 {
		return append([]projectDef(nil), allProjects...), nil
	}

	selected := make([]projectDef, 0, len(projectNames))
	seen := make(map[string]struct{})
	unknown := make([]string, 0)

	for _, rawName := range projectNames {
		name := strings.TrimSpace(rawName)
		if name == "" {
			continue
		}

		normalized := filepath.ToSlash(strings.TrimPrefix(name, "./"))
		matched := false
		for _, project := range allProjects {
			if normalized != project.Name && normalized != filepath.ToSlash(project.DirRelPath) {
				continue
			}

			if _, exists := seen[project.Name]; !exists {
				selected = append(selected, project)
				seen[project.Name] = struct{}{}
			}
			matched = true
			break
		}

		if !matched {
			unknown = append(unknown, name)
		}
	}

	if len(unknown) > 0 {
		return nil, fmt.Errorf("unknown project(s): %s", strings.Join(uniqueSortedStrings(unknown), ", "))
	}

	if len(selected) == 0 {
		return nil, errors.New("no valid project selected")
	}

	return selected, nil
}

func runQuickReview(repoRoot string, projects []projectDef, fix bool) error {
	for _, project := range projects {
		fmt.Printf("[quick-review] project: %s\n", project.Name)
		manifestPath := filepath.Join(repoRoot, project.CargoToml)

		fmtArgs := []string{"fmt"}
		if !fix {
			fmtArgs = append(fmtArgs, "--check")
		}
		fmtArgs = append(fmtArgs, "--manifest-path", manifestPath)
		if err := runCommand(repoRoot, "cargo", fmtArgs, nil); err != nil {
			return fmt.Errorf("format check failed for %s: %w", project.Name, err)
		}

		clippyArgs := []string{
			"clippy",
			"--manifest-path", manifestPath,
			"--all-targets",
			"--",
			"-D", "warnings",
		}
		if err := runCommand(repoRoot, "cargo", clippyArgs, nil); err != nil {
			return fmt.Errorf("clippy failed for %s: %w", project.Name, err)
		}
	}

	return nil
}

func buildManagedHookScript(hookName string) (string, error) {
	header := "#!/bin/sh\n# managed-by-rusktask\nset -eu\nrepo_root=\"$(git rev-parse --show-toplevel)\"\ncd \"$repo_root/scripts/go/rusktask\"\n"

	switch hookName {
	case "pre-commit":
		return header +
			"go run . git-check\n" +
			"go run . workspace-cleanup --mode audit --strict\n" +
			"go run . workspace-guard --mode audit --strict --use-staged-paths\n", nil
	case "pre-push":
		return header +
			"go run . workspace-cleanup --mode audit --strict\n" +
			"go run . workspace-guard --mode audit --strict --use-staged-paths\n" +
			"go run . git-check --pre-push\n" +
			"go run . review --before-push\n", nil
	case "commit-msg":
		return header +
			"msg_file=\"${1:-}\"\n" +
			"if [ -z \"$msg_file\" ]; then\n" +
			"  echo \"commit message file path is required\" >&2\n" +
			"  exit 1\n" +
			"fi\n" +
			"go run . git-check --commit-msg-file \"$msg_file\"\n", nil
	default:
		return "", fmt.Errorf("unsupported hook name: %s", hookName)
	}
}

func runReleaseSync(args []string) int {
	fs := flag.NewFlagSet("release-sync", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	mode := fs.String("mode", "audit", "Mode: audit or apply")
	prune := fs.Bool("prune-local-tags-not-on-remote", false, "Prune local tags not present on remote")
	cleanOrphanNotes := fs.Bool("clean-orphan-notes", false, "Clean orphan release notes")
	skipRemote := fs.Bool("skip-remote", false, "Skip fetching and checking remote tags")
	strict := fs.Bool("strict", false, "Fail when inconsistencies are found")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if *mode != "audit" && *mode != "apply" {
		fmt.Fprintln(os.Stderr, "--mode must be audit or apply")
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	snapshot, err := collectReleaseState(repoRoot, *skipRemote)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	printReleaseStateSummary(snapshot)

	if *mode == "apply" {
		if *prune && len(snapshot.RemoteTags) > 0 {
			for _, tag := range snapshot.LocalOnlyTags {
				if err := deleteLocalTag(repoRoot, tag); err != nil {
					fmt.Fprintf(os.Stderr, "%v\n", err)
					return exitExecution
				}
				fmt.Printf("Deleted local-only tag: %s\n", tag)
			}
		}

		if *cleanOrphanNotes {
			refreshed, err := collectReleaseState(repoRoot, true)
			if err != nil {
				fmt.Fprintf(os.Stderr, "%v\n", err)
				return exitExecution
			}

			for _, tag := range refreshed.OrphanNotes {
				notePath, exists := refreshed.NoteMap[tag]
				if !exists {
					continue
				}

				if err := os.Remove(notePath); err != nil {
					fmt.Fprintf(os.Stderr, "failed to delete orphan release note RELEASE_NOTES_%s.md: %v\n", tag, err)
					return exitExecution
				}
				fmt.Printf("Deleted orphan release note: RELEASE_NOTES_%s.md\n", tag)
			}
		}

		if err := updateReleaseIndex(repoRoot, false); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		snapshot, err = collectReleaseState(repoRoot, *skipRemote)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}
	}

	issueCount := len(snapshot.LocalOnlyTags) + len(snapshot.OrphanNotes) + len(snapshot.OrphanTags)
	if *strict && issueCount > 0 {
		return exitUsage
	}

	fmt.Printf("Release state %s completed.\n", *mode)

	return exitSuccess
}

func runWorkflowSeal(args []string) int {
	fs := flag.NewFlagSet("workflow-seal", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	mode := fs.String("mode", "audit", "Mode: audit or apply")
	prune := fs.Bool("prune-local-tags-not-on-remote", false, "Prune local tags not present on remote")
	cleanOrphanNotes := fs.Bool("clean-orphan-notes", false, "Clean orphan release notes")
	skipRemote := fs.Bool("skip-remote", false, "Skip fetching and checking remote tags")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if *mode != "audit" && *mode != "apply" {
		fmt.Fprintln(os.Stderr, "--mode must be audit or apply")
		return exitUsage
	}

	fmt.Printf("Workflow seal started. Mode: %s\n", *mode)

	if *mode == "audit" {
		if code := runWorkspaceCleanup([]string{"--mode", "audit", "--strict"}); code != exitSuccess {
			return code
		}

		if code := runWorkspaceGuard([]string{"--mode", "audit", "--strict"}); code != exitSuccess {
			return code
		}

		syncArgs := []string{"--mode", "audit", "--strict"}
		if *skipRemote {
			syncArgs = append(syncArgs, "--skip-remote")
		}
		if code := runReleaseSync(syncArgs); code != exitSuccess {
			return code
		}

		repoRoot, code := requireRepoRoot()
		if code != exitSuccess {
			return code
		}

		if err := updateReleaseIndex(repoRoot, false); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		fmt.Println("Workflow seal audit completed.")
		return exitSuccess
	}

	if code := runWorkspaceCleanup([]string{"--mode", "apply"}); code != exitSuccess {
		return code
	}

	if code := runWorkspaceGuard([]string{"--mode", "apply", "--strict"}); code != exitSuccess {
		return code
	}

	syncApplyArgs := []string{"--mode", "apply"}
	if *prune {
		syncApplyArgs = append(syncApplyArgs, "--prune-local-tags-not-on-remote")
	}
	if *cleanOrphanNotes {
		syncApplyArgs = append(syncApplyArgs, "--clean-orphan-notes")
	}
	if *skipRemote {
		syncApplyArgs = append(syncApplyArgs, "--skip-remote")
	}
	if code := runReleaseSync(syncApplyArgs); code != exitSuccess {
		return code
	}

	if code := runWorkspaceCleanup([]string{"--mode", "apply"}); code != exitSuccess {
		return code
	}

	if code := runWorkspaceGuard([]string{"--mode", "audit", "--strict"}); code != exitSuccess {
		return code
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	if err := updateReleaseIndex(repoRoot, false); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	fmt.Println("Workflow seal apply completed.")

	return exitSuccess
}

func runUpdateReleaseIndexCommand(args []string) int {
	fs := flag.NewFlagSet("update-release-index", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	skipRemote := fs.Bool("skip-remote", false, "Skip fetching and checking remote tags")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	if err := updateReleaseIndex(repoRoot, *skipRemote); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	return exitSuccess
}

func runPackageWindowsInstaller(args []string) int {
	fs := flag.NewFlagSet("package-windows-installer", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	version := fs.String("version", "", "Release version (without v prefix)")
	buildTag := fs.String("build-tag", "", "Build tag for iExpress fallback artifact name")
	preferIExpress := fs.Bool("prefer-iexpress", false, "Prefer iExpress packaging even when ISCC is available")
	skipBuild := fs.Bool("skip-build", false, "Skip cargo build in installer packaging")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}
	projectRoot := filepath.Join(repoRoot, "robot_control_rust")
	mainManifestPath := filepath.Join(projectRoot, "Cargo.toml")
	suiteManifestPath := filepath.Join(repoRoot, filepath.FromSlash("rust_tools_suite/Cargo.toml"))
	mainExe := filepath.Join(projectRoot, filepath.FromSlash("target/release/robot_control_rust.exe"))
	suiteExe := filepath.Join(repoRoot, filepath.FromSlash("rust_tools_suite/target/release/rust_tools_suite.exe"))
	archDoc := filepath.Join(projectRoot, "ARCHITECTURE_AND_USAGE.md")
	issPath := filepath.Join(projectRoot, filepath.FromSlash("installer/robot_control_rust_x64.iss"))
	stageDir := filepath.Join(projectRoot, filepath.FromSlash("dist/windows-x64/stage"))
	outputDir := filepath.Join(projectRoot, filepath.FromSlash("dist/windows-x64/installer"))

	resolvedVersion, err := resolveReleaseVersion(mainManifestPath, *version)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	if !*skipBuild {
		if err := buildReleaseBinaries(repoRoot, mainManifestPath, suiteManifestPath); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}
	}

	for _, required := range []string{mainExe, suiteExe, archDoc, issPath} {
		if !fileExists(required) {
			fmt.Fprintf(os.Stderr, "required file not found: %s\n", required)
			return exitExecution
		}
	}

	if err := removeIfExists(stageDir); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := os.MkdirAll(stageDir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create stage dir: %v\n", err)
		return exitExecution
	}
	if err := os.MkdirAll(outputDir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create output dir: %v\n", err)
		return exitExecution
	}

	if err := copyFile(mainExe, filepath.Join(stageDir, "robot_control_rust.exe")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyFile(suiteExe, filepath.Join(stageDir, "rust_tools_suite.exe")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyFile(archDoc, filepath.Join(stageDir, "ARCHITECTURE_AND_USAGE.md")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := packageDocsBundle(repoRoot, stageDir, false); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	isccPath := ""
	if !*preferIExpress {
		isccPath = findISCCExecutable()
	}
	if isccPath != "" {
		isccArgs := []string{
			fmt.Sprintf("/DAppVersion=%s", resolvedVersion),
			fmt.Sprintf("/DProjectRoot=%s", projectRoot),
			fmt.Sprintf("/DStageDir=%s", stageDir),
			fmt.Sprintf("/DOutputDir=%s", outputDir),
			issPath,
		}

		if err := runCommand(repoRoot, isccPath, isccArgs, nil); err != nil {
			fmt.Fprintf(os.Stderr, "ISCC failed: %v\n", err)
			return exitExecution
		}

		pattern := filepath.Join(outputDir, fmt.Sprintf("*%s*_x64_Setup.exe", resolvedVersion))
		installerPath, err := newestFileByGlob(pattern)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		info, err := os.Stat(installerPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "failed to stat installer: %v\n", err)
			return exitExecution
		}

		fmt.Println("[Package] Success")
		fmt.Printf("[Package] Installer: %s\n", installerPath)
		fmt.Printf("[Package] Size MB: %.2f\n", float64(info.Size())/(1024.0*1024.0))
		return exitSuccess
	}

	fmt.Fprintln(os.Stderr, "[Package] Inno Setup not found. Falling back to iExpress...")
	installerPath, err := packageWindowsInstallerIExpress(repoRoot, projectRoot, resolvedVersion, strings.TrimSpace(*buildTag), mainExe, suiteExe, archDoc, outputDir, stageDir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	info, err := os.Stat(installerPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to stat installer: %v\n", err)
		return exitExecution
	}

	fmt.Println("[IExpressPackage] Success")
	fmt.Printf("[IExpressPackage] Installer: %s\n", installerPath)
	fmt.Printf("[IExpressPackage] Size MB: %.2f\n", float64(info.Size())/(1024.0*1024.0))

	return exitSuccess
}

func runPackageWindowsAssets(args []string) int {
	fs := flag.NewFlagSet("package-windows-assets", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	version := fs.String("version", "", "Release version (without v prefix)")
	outputDir := fs.String("output-dir", "", "Output directory for packaged assets")
	skipBuild := fs.Bool("skip-build", false, "Skip cargo build in assets packaging")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}
	projectRoot := filepath.Join(repoRoot, "robot_control_rust")
	mainManifestPath := filepath.Join(projectRoot, "Cargo.toml")
	suiteManifestPath := filepath.Join(repoRoot, filepath.FromSlash("rust_tools_suite/Cargo.toml"))
	mainExe := filepath.Join(projectRoot, filepath.FromSlash("target/release/robot_control_rust.exe"))
	suiteExe := filepath.Join(repoRoot, filepath.FromSlash("rust_tools_suite/target/release/rust_tools_suite.exe"))
	archDoc := filepath.Join(projectRoot, "ARCHITECTURE_AND_USAGE.md")
	suiteReadme := filepath.Join(repoRoot, filepath.FromSlash("rust_tools_suite/README.md"))
	tempRoot := filepath.Join(projectRoot, filepath.FromSlash("dist/windows-x64/release-assets-tmp"))
	docsRoot := filepath.Join(tempRoot, "docs-root")
	mainBundleRoot := filepath.Join(tempRoot, "robot_control_rust")
	suiteBundleRoot := filepath.Join(tempRoot, "rust_tools_suite")

	resolvedVersion, err := resolveReleaseVersion(mainManifestPath, *version)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	resolvedOutputDir := resolveOutputDir(repoRoot, *outputDir, "release_artifacts")

	if !*skipBuild {
		if err := buildReleaseBinaries(repoRoot, mainManifestPath, suiteManifestPath); err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}
	}

	for _, required := range []string{mainExe, suiteExe, archDoc, suiteReadme} {
		if !fileExists(required) {
			fmt.Fprintf(os.Stderr, "required file not found: %s\n", required)
			return exitExecution
		}
	}

	if err := removeIfExists(tempRoot); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	defer func() {
		_ = os.RemoveAll(tempRoot)
	}()

	if err := os.MkdirAll(resolvedOutputDir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create output dir: %v\n", err)
		return exitExecution
	}
	if err := os.MkdirAll(docsRoot, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create docs root: %v\n", err)
		return exitExecution
	}

	if err := packageDocsBundle(repoRoot, docsRoot, false); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	mainZip := filepath.Join(resolvedOutputDir, fmt.Sprintf("robot_control_rust_%s_windows_x64_portable.zip", resolvedVersion))
	suiteZip := filepath.Join(resolvedOutputDir, fmt.Sprintf("rust_tools_suite_%s_windows_x64_portable.zip", resolvedVersion))

	if err := removeIfExists(mainZip); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := removeIfExists(suiteZip); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	if err := os.MkdirAll(mainBundleRoot, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create main bundle dir: %v\n", err)
		return exitExecution
	}
	if err := os.MkdirAll(suiteBundleRoot, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create suite bundle dir: %v\n", err)
		return exitExecution
	}

	if err := copyFile(mainExe, filepath.Join(mainBundleRoot, "robot_control_rust.exe")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyFile(archDoc, filepath.Join(mainBundleRoot, "ARCHITECTURE_AND_USAGE.md")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyFile(filepath.Join(docsRoot, "help_index.html"), filepath.Join(mainBundleRoot, "help_index.html")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyDir(filepath.Join(docsRoot, "docs"), filepath.Join(mainBundleRoot, "docs")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	if err := copyFile(suiteExe, filepath.Join(suiteBundleRoot, "rust_tools_suite.exe")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyFile(suiteReadme, filepath.Join(suiteBundleRoot, "README.md")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyFile(filepath.Join(docsRoot, "help_index.html"), filepath.Join(suiteBundleRoot, "help_index.html")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyDir(filepath.Join(docsRoot, "docs"), filepath.Join(suiteBundleRoot, "docs")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	if err := zipDirContents(mainBundleRoot, mainZip); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create %s: %v\n", mainZip, err)
		return exitExecution
	}
	if err := zipDirContents(suiteBundleRoot, suiteZip); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create %s: %v\n", suiteZip, err)
		return exitExecution
	}

	fmt.Println("[PackageAssets] Success")
	fmt.Printf("[PackageAssets] Main zip: %s\n", mainZip)
	fmt.Printf("[PackageAssets] Suite zip: %s\n", suiteZip)

	return exitSuccess
}

func runDocsBundle(args []string) int {
	fs := flag.NewFlagSet("docs-bundle", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	outputRoot := fs.String("output-root", "", "Output root for docs bundle")
	createZip := fs.Bool("create-zip", false, "Create docs_bundle.zip in output root")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	resolvedOutputRoot := strings.TrimSpace(*outputRoot)
	if resolvedOutputRoot == "" {
		resolvedOutputRoot = filepath.Join(repoRoot, filepath.FromSlash("robot_control_rust/dist/windows-x64/docs-bundle"))
	} else if !filepath.IsAbs(resolvedOutputRoot) {
		resolvedOutputRoot = filepath.Join(repoRoot, filepath.FromSlash(resolvedOutputRoot))
	}

	if err := packageDocsBundle(repoRoot, resolvedOutputRoot, *createZip); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	fmt.Println("[DocsBundle] Success")
	fmt.Printf("[DocsBundle] Output root: %s\n", resolvedOutputRoot)
	if *createZip {
		fmt.Printf("[DocsBundle] Zip: %s\n", filepath.Join(resolvedOutputRoot, "docs_bundle.zip"))
	}

	return exitSuccess
}

func runReleasePublish(args []string) int {
	fs := flag.NewFlagSet("release-publish", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	owner := fs.String("owner", "loopgap", "GitHub owner")
	repo := fs.String("repo", "robot_ctrl_rust_app", "GitHub repository")
	tag := fs.String("tag", "", "Release tag, e.g. v0.1.7")
	releaseName := fs.String("release-name", "", "Release display name")
	bodyFile := fs.String("body-file", "", "Release notes file path")
	prerelease := fs.Bool("prerelease", false, "Mark release as prerelease")
	draft := fs.Bool("draft", false, "Create release as draft")
	pruneExtraAssets := fs.Bool("prune-extra-assets", false, "Delete assets that are not in --asset list")
	var assets stringSliceFlag
	fs.Var(&assets, "asset", "Asset file path to upload (repeatable)")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	trimmedTag := strings.TrimSpace(*tag)
	if trimmedTag == "" {
		fmt.Fprintln(os.Stderr, "--tag is required (example: v0.1.7)")
		return exitUsage
	}

	trimmedOwner := strings.TrimSpace(*owner)
	trimmedRepo := strings.TrimSpace(*repo)
	if trimmedOwner == "" || trimmedRepo == "" {
		fmt.Fprintln(os.Stderr, "--owner and --repo must not be empty")
		return exitUsage
	}

	token := strings.TrimSpace(os.Getenv("GITHUB_TOKEN"))
	if token == "" {
		fmt.Fprintln(os.Stderr, "missing GITHUB_TOKEN environment variable")
		return exitPrecondition
	}

	effectiveReleaseName := strings.TrimSpace(*releaseName)
	if effectiveReleaseName == "" {
		effectiveReleaseName = trimmedTag
	}

	effectiveBodyFile := strings.TrimSpace(*bodyFile)
	if effectiveBodyFile == "" {
		effectiveBodyFile = filepath.Join(repoRoot, "release_notes", fmt.Sprintf("RELEASE_NOTES_%s.md", trimmedTag))
	} else if !filepath.IsAbs(effectiveBodyFile) {
		effectiveBodyFile = filepath.Join(repoRoot, filepath.FromSlash(effectiveBodyFile))
	}

	bodyBytes, err := os.ReadFile(effectiveBodyFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to read release notes file %s: %v\n", effectiveBodyFile, err)
		return exitExecution
	}

	effectiveAssets := make([]string, 0)
	if len(assets) == 0 {
		effectiveAssets = []string{
			filepath.Join(repoRoot, filepath.FromSlash("release_artifacts/robot_control_rust_windows_x64_portable.zip")),
			filepath.Join(repoRoot, filepath.FromSlash("release_artifacts/rust_tools_suite_windows_x64_portable.zip")),
			filepath.Join(repoRoot, filepath.FromSlash("release_artifacts/rust_tools_suite_linux_amd64.deb")),
			filepath.Join(repoRoot, filepath.FromSlash("release_artifacts/RobotControlSuite_Setup.exe")),
			filepath.Join(repoRoot, filepath.FromSlash("release_artifacts/checksums-sha256.txt")),
		}
	} else {
		for _, asset := range assets {
			trimmed := strings.TrimSpace(asset)
			if trimmed == "" {
				continue
			}
			if !filepath.IsAbs(trimmed) {
				trimmed = filepath.Join(repoRoot, filepath.FromSlash(trimmed))
			}
			effectiveAssets = append(effectiveAssets, filepath.Clean(trimmed))
		}
	}

	if len(effectiveAssets) == 0 {
		fmt.Fprintln(os.Stderr, "no release assets configured")
		return exitUsage
	}

	for _, asset := range effectiveAssets {
		if !fileExists(asset) {
			fmt.Fprintf(os.Stderr, "asset not found: %s\n", asset)
			return exitPrecondition
		}
	}

	createPayload := map[string]any{
		"tag_name":   trimmedTag,
		"name":       effectiveReleaseName,
		"body":       string(bodyBytes),
		"draft":      *draft,
		"prerelease": *prerelease,
	}

	createBody, err := json.Marshal(createPayload)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to marshal release payload: %v\n", err)
		return exitExecution
	}

	createURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases", trimmedOwner, trimmedRepo)
	body, statusCode, reqErr := doGitHubRequest("POST", createURL, token, createBody, "application/json; charset=utf-8")

	release := githubRelease{}
	if reqErr != nil {
		if statusCode != http.StatusUnprocessableEntity {
			fmt.Fprintf(os.Stderr, "failed to create release: %v\n", reqErr)
			return exitExecution
		}

		getURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases/tags/%s", trimmedOwner, trimmedRepo, url.PathEscape(trimmedTag))
		getBody, _, getErr := doGitHubRequest("GET", getURL, token, nil, "")
		if getErr != nil {
			fmt.Fprintf(os.Stderr, "failed to load existing release for tag %s: %v\n", trimmedTag, getErr)
			return exitExecution
		}

		if err := json.Unmarshal(getBody, &release); err != nil {
			fmt.Fprintf(os.Stderr, "failed to parse existing release response: %v\n", err)
			return exitExecution
		}

		updatePayload := map[string]any{
			"name":       effectiveReleaseName,
			"body":       string(bodyBytes),
			"draft":      *draft,
			"prerelease": *prerelease,
		}
		updateBody, err := json.Marshal(updatePayload)
		if err != nil {
			fmt.Fprintf(os.Stderr, "failed to marshal release update payload: %v\n", err)
			return exitExecution
		}

		updateURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases/%d", trimmedOwner, trimmedRepo, release.ID)
		patchedBody, _, patchErr := doGitHubRequest("PATCH", updateURL, token, updateBody, "application/json; charset=utf-8")
		if patchErr != nil {
			fmt.Fprintf(os.Stderr, "failed to update existing release: %v\n", patchErr)
			return exitExecution
		}

		if err := json.Unmarshal(patchedBody, &release); err != nil {
			fmt.Fprintf(os.Stderr, "failed to parse updated release response: %v\n", err)
			return exitExecution
		}
	} else {
		if err := json.Unmarshal(body, &release); err != nil {
			fmt.Fprintf(os.Stderr, "failed to parse created release response: %v\n", err)
			return exitExecution
		}
	}

	desiredByName := make(map[string]string)
	for _, asset := range effectiveAssets {
		desiredByName[filepath.Base(asset)] = asset
	}

	if *pruneExtraAssets {
		for _, existing := range release.Assets {
			if _, ok := desiredByName[existing.Name]; ok {
				continue
			}

			deleteURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases/assets/%d", trimmedOwner, trimmedRepo, existing.ID)
			if _, _, delErr := doGitHubRequest("DELETE", deleteURL, token, nil, ""); delErr != nil {
				fmt.Fprintf(os.Stderr, "failed to delete extra asset %s: %v\n", existing.Name, delErr)
				return exitExecution
			}
		}
	}

	for _, assetPath := range effectiveAssets {
		assetName := filepath.Base(assetPath)

		for _, existing := range release.Assets {
			if existing.Name != assetName {
				continue
			}

			deleteURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/releases/assets/%d", trimmedOwner, trimmedRepo, existing.ID)
			if _, _, delErr := doGitHubRequest("DELETE", deleteURL, token, nil, ""); delErr != nil {
				fmt.Fprintf(os.Stderr, "failed to delete existing asset %s: %v\n", assetName, delErr)
				return exitExecution
			}
		}

		baseUploadURL := strings.ReplaceAll(release.UploadURL, "{?name,label}", "")
		uploadURL := fmt.Sprintf("%s?name=%s", baseUploadURL, url.QueryEscape(assetName))

		assetBytes, readErr := os.ReadFile(assetPath)
		if readErr != nil {
			fmt.Fprintf(os.Stderr, "failed to read asset %s: %v\n", assetPath, readErr)
			return exitExecution
		}

		if _, _, uploadErr := doGitHubRequest("POST", uploadURL, token, assetBytes, "application/octet-stream"); uploadErr != nil {
			fmt.Fprintf(os.Stderr, "failed to upload asset %s: %v\n", assetPath, uploadErr)
			return exitExecution
		}

		fmt.Printf("Uploaded asset: %s\n", assetName)
	}

	fmt.Printf("Release published: %s\n", release.HTMLURL)
	return exitSuccess
}

func runBuildReleaseSlim(args []string) int {
	fs := flag.NewFlagSet("build-release-slim", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	skipTests := fs.Bool("skip-tests", false, "Skip cargo test and cargo test --release")
	skipClippy := fs.Bool("skip-clippy", false, "Skip cargo clippy checks")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	projectRoot := filepath.Join(repoRoot, "robot_control_rust")
	manifestPath := filepath.Join(projectRoot, "Cargo.toml")
	targetDir := filepath.Join(projectRoot, "target")

	if !fileExists(manifestPath) {
		fmt.Fprintf(os.Stderr, "manifest not found: %s\n", manifestPath)
		return exitPrecondition
	}

	beforeBytes, _ := directorySize(targetDir)
	fmt.Printf("[ReleaseSlim] Target size before: %d bytes\n", beforeBytes)

	if err := runCommand(repoRoot, "cargo", []string{"fmt", "--check", "--manifest-path", manifestPath}, nil); err != nil {
		return exitExecution
	}

	if !*skipClippy {
		if err := runCommand(repoRoot, "cargo", []string{"clippy", "--manifest-path", manifestPath, "--all-targets", "--", "-D", "warnings"}, nil); err != nil {
			return exitExecution
		}
	}

	if !*skipTests {
		if err := runCommand(repoRoot, "cargo", []string{"test", "--manifest-path", manifestPath}, nil); err != nil {
			return exitExecution
		}
		if err := runCommand(repoRoot, "cargo", []string{"test", "--release", "--manifest-path", manifestPath}, nil); err != nil {
			return exitExecution
		}
	}

	cleanupTargets := []string{
		filepath.Join(projectRoot, filepath.FromSlash("target/debug")),
		filepath.Join(projectRoot, filepath.FromSlash("target/flycheck0")),
		filepath.Join(projectRoot, filepath.FromSlash("target/release/deps")),
		filepath.Join(projectRoot, filepath.FromSlash("target/release/build")),
		filepath.Join(projectRoot, filepath.FromSlash("target/release/incremental")),
		filepath.Join(projectRoot, filepath.FromSlash("target/release/examples")),
	}

	for _, cleanupTarget := range cleanupTargets {
		if err := removeIfExists(cleanupTarget); err != nil {
			fmt.Fprintf(os.Stderr, "warning: failed to remove %s: %v\n", cleanupTarget, err)
		}
	}

	if err := runCommand(repoRoot, "cargo", []string{"build", "--release", "--manifest-path", manifestPath}, nil); err != nil {
		return exitExecution
	}

	for _, cleanupTarget := range cleanupTargets {
		if err := removeIfExists(cleanupTarget); err != nil {
			fmt.Fprintf(os.Stderr, "warning: failed to remove %s: %v\n", cleanupTarget, err)
		}
	}

	afterBytes, _ := directorySize(targetDir)
	fmt.Printf("[ReleaseSlim] Target size after : %d bytes\n", afterBytes)
	if beforeBytes > 0 {
		delta := beforeBytes - afterBytes
		ratio := (float64(afterBytes) / float64(beforeBytes)) * 100.0
		fmt.Printf("[ReleaseSlim] Reduced by %d bytes, remaining %.2f%%\n", delta, ratio)
	}

	releaseBin := filepath.Join(projectRoot, filepath.FromSlash("target/release/robot_control_rust.exe"))
	if !fileExists(releaseBin) {
		releaseBin = filepath.Join(projectRoot, filepath.FromSlash("target/release/robot_control_rust"))
	}
	if !fileExists(releaseBin) {
		fmt.Fprintln(os.Stderr, "[ReleaseSlim] Release binary not found")
		return exitExecution
	}

	info, statErr := os.Stat(releaseBin)
	if statErr != nil {
		fmt.Fprintf(os.Stderr, "[ReleaseSlim] Failed to stat release binary: %v\n", statErr)
		return exitExecution
	}

	fmt.Printf("[ReleaseSlim] Release binary: %s\n", releaseBin)
	fmt.Printf("[ReleaseSlim] Binary size  : %d bytes\n", info.Size())
	fmt.Printf("[ReleaseSlim] LastWriteTime: %s\n", info.ModTime().Format(time.RFC3339))
	fmt.Println("[ReleaseSlim] Done")

	return exitSuccess
}

func runPackageWindowsPortableInstaller(args []string) int {
	fs := flag.NewFlagSet("package-windows-portable-installer", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	version := fs.String("version", "", "Release version (without v prefix)")
	outputDir := fs.String("output-dir", "", "Output directory for installer bundle zip")
	skipBuild := fs.Bool("skip-build", false, "Skip cargo build in packaging")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	projectRoot := filepath.Join(repoRoot, "robot_control_rust")
	manifestPath := filepath.Join(projectRoot, "Cargo.toml")
	releaseExe := filepath.Join(projectRoot, filepath.FromSlash("target/release/robot_control_rust.exe"))
	if !fileExists(releaseExe) {
		releaseExe = filepath.Join(projectRoot, filepath.FromSlash("target/release/robot_control_rust"))
	}
	archDoc := filepath.Join(projectRoot, "ARCHITECTURE_AND_USAGE.md")

	resolvedVersion, err := resolveReleaseVersion(manifestPath, *version)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	if !*skipBuild {
		if err := runCommand(repoRoot, "cargo", []string{"build", "--release", "--manifest-path", manifestPath}, nil); err != nil {
			return exitExecution
		}
	}

	if !fileExists(releaseExe) {
		fmt.Fprintf(os.Stderr, "release executable not found: %s\n", releaseExe)
		return exitPrecondition
	}
	if !fileExists(archDoc) {
		fmt.Fprintf(os.Stderr, "required file not found: %s\n", archDoc)
		return exitPrecondition
	}

	distRoot := filepath.Join(projectRoot, filepath.FromSlash("dist/windows-x64"))
	bundleDir := filepath.Join(distRoot, "installer-bundle")
	resolvedOutputDir := strings.TrimSpace(*outputDir)
	if resolvedOutputDir == "" {
		resolvedOutputDir = filepath.Join(distRoot, "installer")
	} else if !filepath.IsAbs(resolvedOutputDir) {
		resolvedOutputDir = filepath.Join(repoRoot, filepath.FromSlash(resolvedOutputDir))
	}

	if err := removeIfExists(bundleDir); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := os.MkdirAll(bundleDir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create bundle dir: %v\n", err)
		return exitExecution
	}
	if err := os.MkdirAll(resolvedOutputDir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create output dir: %v\n", err)
		return exitExecution
	}

	if err := copyFile(releaseExe, filepath.Join(bundleDir, "robot_control_rust.exe")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := copyFile(archDoc, filepath.Join(bundleDir, "ARCHITECTURE_AND_USAGE.md")); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	installCmdPath := filepath.Join(bundleDir, "Install_RobotControlSuite_x64.cmd")
	uninstallCmdPath := filepath.Join(bundleDir, "Uninstall_RobotControlSuite_x64.cmd")

	installCmdContent := "@echo off\r\n" +
		"setlocal\r\n" +
		"set \"TARGET=%LOCALAPPDATA%\\Robot Control Suite\"\r\n" +
		"if not exist \"%TARGET%\" mkdir \"%TARGET%\"\r\n" +
		"copy /Y \"%~dp0robot_control_rust.exe\" \"%TARGET%\\robot_control_rust.exe\" >nul\r\n" +
		"copy /Y \"%~dp0ARCHITECTURE_AND_USAGE.md\" \"%TARGET%\\ARCHITECTURE_AND_USAGE.md\" >nul\r\n" +
		"powershell -NoProfile -ExecutionPolicy Bypass -Command \"$s=(New-Object -ComObject WScript.Shell); $lnk=$s.CreateShortcut([System.IO.Path]::Combine($env:USERPROFILE,'Desktop','Robot Control Suite.lnk')); $lnk.TargetPath=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite','robot_control_rust.exe'); $lnk.WorkingDirectory=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite'); $lnk.Save();\"\r\n" +
		"powershell -NoProfile -ExecutionPolicy Bypass -Command \"$sm=[Environment]::GetFolderPath('StartMenu'); $dir=Join-Path $sm 'Programs'; $s=(New-Object -ComObject WScript.Shell); $lnk=$s.CreateShortcut((Join-Path $dir 'Robot Control Suite.lnk')); $lnk.TargetPath=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite','robot_control_rust.exe'); $lnk.WorkingDirectory=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite'); $lnk.Save();\"\r\n" +
		"echo Installed to: %TARGET%\r\n" +
		"start \"\" \"%TARGET%\\robot_control_rust.exe\"\r\n" +
		"exit /b 0\r\n"

	uninstallCmdContent := "@echo off\r\n" +
		"setlocal\r\n" +
		"set \"TARGET=%LOCALAPPDATA%\\Robot Control Suite\"\r\n" +
		"del /F /Q \"%TARGET%\\robot_control_rust.exe\" 2>nul\r\n" +
		"del /F /Q \"%TARGET%\\ARCHITECTURE_AND_USAGE.md\" 2>nul\r\n" +
		"del /F /Q \"%TARGET%\\Uninstall_RobotControlSuite_x64.cmd\" 2>nul\r\n" +
		"rmdir \"%TARGET%\" 2>nul\r\n" +
		"del /F /Q \"%USERPROFILE%\\Desktop\\Robot Control Suite.lnk\" 2>nul\r\n" +
		"del /F /Q \"%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\Robot Control Suite.lnk\" 2>nul\r\n" +
		"echo Uninstall completed.\r\n" +
		"exit /b 0\r\n"

	if err := writeTextFile(installCmdPath, installCmdContent, 0o644); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}
	if err := writeTextFile(uninstallCmdPath, uninstallCmdContent, 0o644); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	outputZip := filepath.Join(resolvedOutputDir, fmt.Sprintf("RobotControlSuite_%s_x64_InstallerBundle.zip", resolvedVersion))
	if err := removeIfExists(outputZip); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	if err := zipDirContents(bundleDir, outputZip); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create installer bundle zip: %v\n", err)
		return exitExecution
	}

	info, err := os.Stat(outputZip)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to stat installer bundle zip: %v\n", err)
		return exitExecution
	}

	fmt.Println("[PortablePackage] Success")
	fmt.Printf("[PortablePackage] Installer: %s\n", outputZip)
	fmt.Printf("[PortablePackage] Size MB: %.2f\n", float64(info.Size())/(1024.0*1024.0))

	return exitSuccess
}

func doGitHubRequest(method string, requestURL string, token string, body []byte, contentType string) ([]byte, int, error) {
	var reader io.Reader
	if body != nil {
		reader = bytes.NewReader(body)
	}

	req, err := http.NewRequest(method, requestURL, reader)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to create GitHub request: %w", err)
	}

	req.Header.Set("Authorization", "Bearer "+token)
	req.Header.Set("Accept", "application/vnd.github+json")
	req.Header.Set("X-GitHub-Api-Version", "2022-11-28")
	if strings.TrimSpace(contentType) != "" {
		req.Header.Set("Content-Type", contentType)
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, 0, fmt.Errorf("GitHub request failed: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, resp.StatusCode, fmt.Errorf("failed to read GitHub response body: %w", err)
	}

	if resp.StatusCode >= 200 && resp.StatusCode < 300 {
		return respBody, resp.StatusCode, nil
	}

	trimmedResp := strings.TrimSpace(string(respBody))
	if trimmedResp == "" {
		trimmedResp = resp.Status
	}

	return respBody, resp.StatusCode, fmt.Errorf("GitHub API %s %s failed with status %d: %s", method, requestURL, resp.StatusCode, trimmedResp)
}

func directorySize(path string) (int64, error) {
	if !fileExists(path) {
		return 0, nil
	}

	var total int64
	err := filepath.Walk(path, func(current string, info os.FileInfo, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		if info.IsDir() {
			return nil
		}
		total += info.Size()
		return nil
	})

	if err != nil {
		return 0, err
	}

	return total, nil
}

func resolveReleaseVersion(mainManifestPath string, explicitVersion string) (string, error) {
	if strings.TrimSpace(explicitVersion) != "" {
		return strings.TrimSpace(explicitVersion), nil
	}

	content, err := os.ReadFile(mainManifestPath)
	if err != nil {
		return "", fmt.Errorf("failed to read Cargo.toml: %w", err)
	}

	match := regexp.MustCompile(`(?m)^version\s*=\s*"([^"]+)"`).FindStringSubmatch(string(content))
	if len(match) != 2 {
		return "", errors.New("failed to read version from Cargo.toml")
	}

	return strings.TrimSpace(match[1]), nil
}

func resolveOutputDir(repoRoot string, configuredOutputDir string, defaultRelative string) string {
	trimmed := strings.TrimSpace(configuredOutputDir)
	if trimmed == "" {
		return filepath.Join(repoRoot, filepath.FromSlash(defaultRelative))
	}

	if filepath.IsAbs(trimmed) {
		return filepath.Clean(trimmed)
	}

	return filepath.Join(repoRoot, filepath.FromSlash(trimmed))
}

func buildReleaseBinaries(repoRoot string, mainManifestPath string, suiteManifestPath string) error {
	if err := runCommand(repoRoot, "cargo", []string{"build", "--release", "--manifest-path", mainManifestPath}, nil); err != nil {
		return fmt.Errorf("robot_control_rust release build failed: %w", err)
	}

	if err := runCommand(repoRoot, "cargo", []string{"build", "--release", "--manifest-path", suiteManifestPath}, nil); err != nil {
		return fmt.Errorf("rust_tools_suite release build failed: %w", err)
	}

	return nil
}

func packageDocsBundle(repoRoot string, outputRoot string, createZip bool) error {
	docsRoot := filepath.Join(repoRoot, "docs")
	docsOutput := filepath.Join(outputRoot, "docs")
	bookOutput := filepath.Join(docsOutput, "book")
	helpIndexPath := filepath.Join(outputRoot, "help_index.html")
	docsIndexPath := filepath.Join(docsOutput, "index.html")
	docsZipPath := filepath.Join(outputRoot, "docs_bundle.zip")
	localHelpSource := filepath.Join(docsRoot, filepath.FromSlash("help/index.html"))
	bookTomlPath := filepath.Join(docsRoot, "book.toml")

	if !fileExists(bookTomlPath) {
		return fmt.Errorf("mdBook config not found: %s", bookTomlPath)
	}

	mdbookPath, err := exec.LookPath("mdbook")
	if err != nil {
		return errors.New("mdbook was not found in PATH. Install mdbook before packaging release assets")
	}

	if err := removeIfExists(docsOutput); err != nil {
		return err
	}
	if createZip {
		if err := removeIfExists(docsZipPath); err != nil {
			return err
		}
	}

	if err := os.MkdirAll(outputRoot, 0o755); err != nil {
		return fmt.Errorf("failed to create output root: %w", err)
	}
	if err := os.MkdirAll(docsOutput, 0o755); err != nil {
		return fmt.Errorf("failed to create docs output: %w", err)
	}

	if err := runCommand(repoRoot, mdbookPath, []string{"build", docsRoot, "-d", bookOutput}, nil); err != nil {
		return fmt.Errorf("mdbook build failed: %w", err)
	}

	if !fileExists(localHelpSource) {
		return fmt.Errorf("local help source not found: %s", localHelpSource)
	}

	if err := copyFile(localHelpSource, helpIndexPath); err != nil {
		return err
	}
	if err := copyFile(localHelpSource, docsIndexPath); err != nil {
		return err
	}

	if createZip {
		if err := zipDirContents(docsOutput, docsZipPath); err != nil {
			return fmt.Errorf("failed to create docs bundle zip: %w", err)
		}
	}

	return nil
}

func findISCCExecutable() string {
	if path, err := exec.LookPath("ISCC.exe"); err == nil {
		return path
	}

	candidates := []string{
		filepath.Join(os.Getenv("ProgramFiles(x86)"), filepath.FromSlash("Inno Setup 6/ISCC.exe")),
		filepath.Join(os.Getenv("ProgramFiles"), filepath.FromSlash("Inno Setup 6/ISCC.exe")),
		filepath.Join(os.Getenv("LOCALAPPDATA"), filepath.FromSlash("Programs/Inno Setup 6/ISCC.exe")),
		filepath.Join(os.Getenv("LOCALAPPDATA"), filepath.FromSlash("Programs/JRSoftware/Inno Setup 6/ISCC.exe")),
	}

	for _, candidate := range candidates {
		if strings.TrimSpace(candidate) == "" {
			continue
		}
		if fileExists(candidate) {
			return candidate
		}
	}

	return ""
}

func packageWindowsInstallerIExpress(repoRoot string, projectRoot string, version string, buildTag string, mainExe string, suiteExe string, archDoc string, outputDir string, stageDir string) (string, error) {
	iexpressExe := filepath.Join(os.Getenv("WINDIR"), filepath.FromSlash("System32/iexpress.exe"))
	if !fileExists(iexpressExe) {
		return "", fmt.Errorf("IExpress not found: %s", iexpressExe)
	}

	buildTagValue := strings.TrimSpace(buildTag)
	if buildTagValue == "" {
		buildTagValue = time.Now().Format("20060102")
	}

	tempDir := filepath.Join(projectRoot, filepath.FromSlash("dist/windows-x64/iexpress-tmp"))
	if err := removeIfExists(stageDir); err != nil {
		return "", err
	}
	if err := removeIfExists(tempDir); err != nil {
		return "", err
	}

	if err := os.MkdirAll(stageDir, 0o755); err != nil {
		return "", fmt.Errorf("failed to create stage dir: %w", err)
	}
	if err := os.MkdirAll(outputDir, 0o755); err != nil {
		return "", fmt.Errorf("failed to create output dir: %w", err)
	}
	if err := os.MkdirAll(tempDir, 0o755); err != nil {
		return "", fmt.Errorf("failed to create temp dir: %w", err)
	}

	if err := copyFile(mainExe, filepath.Join(stageDir, "robot_control_rust.exe")); err != nil {
		return "", err
	}
	if err := copyFile(suiteExe, filepath.Join(stageDir, "rust_tools_suite.exe")); err != nil {
		return "", err
	}
	if err := copyFile(archDoc, filepath.Join(stageDir, "ARCHITECTURE_AND_USAGE.md")); err != nil {
		return "", err
	}

	if err := packageDocsBundle(repoRoot, stageDir, true); err != nil {
		return "", err
	}

	installCmd := filepath.Join(stageDir, "install.cmd")
	installCmdContent := "@echo off\r\nsetlocal\r\npowershell -NoProfile -ExecutionPolicy Bypass -STA -File \"%~dp0install.ps1\"\r\nif errorlevel 1 exit /b 1\r\nexit /b 0\r\n"
	if err := writeTextFile(installCmd, installCmdContent, 0o644); err != nil {
		return "", err
	}

	installPs1 := filepath.Join(stageDir, "install.ps1")
	installPs1Content := "param(\n    [string]$InstallDir\n)\n\n$ErrorActionPreference = 'Stop'\n\nif ([string]::IsNullOrWhiteSpace($InstallDir)) {\n    $InstallDir = Join-Path $env:LOCALAPPDATA 'Robot Control Suite'\n}\n\nNew-Item -ItemType Directory -Force -Path $InstallDir | Out-Null\nCopy-Item -Force (Join-Path $PSScriptRoot 'robot_control_rust.exe') (Join-Path $InstallDir 'robot_control_rust.exe')\nCopy-Item -Force (Join-Path $PSScriptRoot 'rust_tools_suite.exe') (Join-Path $InstallDir 'rust_tools_suite.exe')\nCopy-Item -Force (Join-Path $PSScriptRoot 'help_index.html') (Join-Path $InstallDir 'help_index.html')\nCopy-Item -Force (Join-Path $PSScriptRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path $InstallDir 'ARCHITECTURE_AND_USAGE.md')\nCopy-Item -Force (Join-Path $PSScriptRoot 'docs_bundle.zip') (Join-Path $InstallDir 'docs_bundle.zip')\nExpand-Archive -LiteralPath (Join-Path $InstallDir 'docs_bundle.zip') -DestinationPath (Join-Path $InstallDir 'docs') -Force\nRemove-Item (Join-Path $InstallDir 'docs_bundle.zip') -Force -ErrorAction SilentlyContinue\nStart-Process (Join-Path $InstallDir 'robot_control_rust.exe')\n"
	if err := writeTextFile(installPs1, installPs1Content, 0o644); err != nil {
		return "", err
	}

	outputExe := filepath.Join(outputDir, fmt.Sprintf("RobotControlSuite_%s_x64_%s_Setup.exe", version, buildTagValue))
	sedPath := filepath.Join(tempDir, "robot_control_suite.sed")
	sedContent := `[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=0
HideExtractAnimation=1
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=
DisplayLicense=
FinishMessage=
TargetName=__TARGET__
FriendlyName=Robot Control Suite __VERSION__
AppLaunched=install.cmd
PostInstallCmd=<None>
AdminQuietInstCmd=install.cmd
UserQuietInstCmd=install.cmd
SourceFiles=SourceFiles
[Strings]
FILE0=install.cmd
FILE1=robot_control_rust.exe
FILE2=rust_tools_suite.exe
FILE3=help_index.html
FILE4=docs_bundle.zip
FILE5=ARCHITECTURE_AND_USAGE.md
FILE6=install.ps1
[SourceFiles]
SourceFiles0=__STAGE__
[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
%FILE3%=
%FILE4%=
%FILE5%=
%FILE6%=
`
	sedContent = strings.ReplaceAll(sedContent, "__TARGET__", outputExe)
	sedContent = strings.ReplaceAll(sedContent, "__VERSION__", version)
	sedContent = strings.ReplaceAll(sedContent, "__STAGE__", stageDir)
	if err := writeTextFile(sedPath, sedContent, 0o644); err != nil {
		return "", err
	}

	if err := removeIfExists(outputExe); err != nil {
		return "", err
	}

	err := runCommand(repoRoot, iexpressExe, []string{"/N", sedPath}, nil)
	if err != nil && !fileExists(outputExe) {
		return "", fmt.Errorf("IExpress failed: %w", err)
	}
	if !fileExists(outputExe) {
		return "", fmt.Errorf("installer not found: %s", outputExe)
	}

	_ = os.RemoveAll(tempDir)
	_ = os.RemoveAll(stageDir)

	return outputExe, nil
}

func newestFileByGlob(pattern string) (string, error) {
	matches, err := filepath.Glob(pattern)
	if err != nil {
		return "", fmt.Errorf("invalid glob pattern %q: %w", pattern, err)
	}
	if len(matches) == 0 {
		return "", fmt.Errorf("no files matched: %s", pattern)
	}

	bestPath := ""
	var bestModTime time.Time
	for _, match := range matches {
		info, statErr := os.Stat(match)
		if statErr != nil {
			continue
		}
		if bestPath == "" || info.ModTime().After(bestModTime) {
			bestPath = match
			bestModTime = info.ModTime()
		}
	}

	if bestPath == "" {
		return "", fmt.Errorf("failed to select newest file from: %s", pattern)
	}

	return bestPath, nil
}

func writeTextFile(path string, content string, perm os.FileMode) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return fmt.Errorf("failed to create directory for %s: %w", path, err)
	}
	if err := os.WriteFile(path, []byte(content), perm); err != nil {
		return fmt.Errorf("failed to write file %s: %w", path, err)
	}
	return nil
}

func removeIfExists(path string) error {
	if !fileExists(path) {
		return nil
	}
	if err := os.RemoveAll(path); err != nil {
		return fmt.Errorf("failed to remove %s: %w", path, err)
	}
	return nil
}

func copyFile(src string, dst string) error {
	info, err := os.Stat(src)
	if err != nil {
		return fmt.Errorf("failed to stat source file %s: %w", src, err)
	}
	if info.IsDir() {
		return fmt.Errorf("source is directory, not file: %s", src)
	}

	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return fmt.Errorf("failed to create destination directory %s: %w", filepath.Dir(dst), err)
	}

	in, err := os.Open(src)
	if err != nil {
		return fmt.Errorf("failed to open source file %s: %w", src, err)
	}
	defer in.Close()

	out, err := os.OpenFile(dst, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, info.Mode())
	if err != nil {
		return fmt.Errorf("failed to open destination file %s: %w", dst, err)
	}
	defer out.Close()

	if _, err := io.Copy(out, in); err != nil {
		return fmt.Errorf("failed to copy %s to %s: %w", src, dst, err)
	}

	return nil
}

func copyDir(src string, dst string) error {
	if !fileExists(src) {
		return fmt.Errorf("source directory not found: %s", src)
	}

	if err := os.MkdirAll(dst, 0o755); err != nil {
		return fmt.Errorf("failed to create destination directory %s: %w", dst, err)
	}

	return filepath.WalkDir(src, func(path string, d os.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}

		relPath, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		if relPath == "." {
			return nil
		}

		target := filepath.Join(dst, relPath)
		if d.IsDir() {
			return os.MkdirAll(target, 0o755)
		}

		return copyFile(path, target)
	})
}

func zipDirContents(srcDir string, destZip string) error {
	if !fileExists(srcDir) {
		return fmt.Errorf("source directory not found: %s", srcDir)
	}

	if err := removeIfExists(destZip); err != nil {
		return err
	}

	if err := os.MkdirAll(filepath.Dir(destZip), 0o755); err != nil {
		return fmt.Errorf("failed to create zip parent dir %s: %w", filepath.Dir(destZip), err)
	}

	zf, err := os.Create(destZip)
	if err != nil {
		return fmt.Errorf("failed to create zip file %s: %w", destZip, err)
	}
	defer zf.Close()

	zWriter := zip.NewWriter(zf)

	if err := filepath.WalkDir(srcDir, func(path string, d os.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		if d.IsDir() {
			return nil
		}

		relPath, err := filepath.Rel(srcDir, path)
		if err != nil {
			return err
		}

		info, err := d.Info()
		if err != nil {
			return err
		}

		header, err := zip.FileInfoHeader(info)
		if err != nil {
			return err
		}
		header.Name = filepath.ToSlash(relPath)
		header.Method = zip.Deflate

		writer, err := zWriter.CreateHeader(header)
		if err != nil {
			return err
		}

		srcFile, err := os.Open(path)
		if err != nil {
			return err
		}

		if _, err := io.Copy(writer, srcFile); err != nil {
			srcFile.Close()
			return err
		}

		if err := srcFile.Close(); err != nil {
			return err
		}

		return nil
	}); err != nil {
		return fmt.Errorf("failed to zip directory %s: %w", srcDir, err)
	}

	if err := zWriter.Close(); err != nil {
		return fmt.Errorf("failed to finalize zip file %s: %w", destZip, err)
	}

	return nil
}

func collectReleaseState(repoRoot string, skipRemote bool) (releaseStateSnapshot, error) {
	snapshot := releaseStateSnapshot{}
	releaseNotesDir := filepath.Join(repoRoot, "release_notes")
	if !fileExists(releaseNotesDir) {
		return snapshot, fmt.Errorf("release_notes directory not found: %s", releaseNotesDir)
	}

	localTags, err := getLocalSemverTags(repoRoot)
	if err != nil {
		return snapshot, err
	}

	remoteTags := []string{}
	if !skipRemote {
		remoteTags = getRemoteSemverTagsForSync(repoRoot)
	}

	noteMap, err := getReleaseNoteMap(releaseNotesDir)
	if err != nil {
		return snapshot, err
	}

	noteTags := uniqueSortedStrings(mapKeys(noteMap))

	localOnlyTags := []string{}
	if len(remoteTags) > 0 {
		for _, tag := range localTags {
			if !containsString(remoteTags, tag) {
				localOnlyTags = append(localOnlyTags, tag)
			}
		}
	}

	orphanNotes := make([]string, 0)
	for _, tag := range noteTags {
		if !containsString(localTags, tag) {
			orphanNotes = append(orphanNotes, tag)
		}
	}

	orphanTags := make([]string, 0)
	for _, tag := range localTags {
		if !containsString(noteTags, tag) {
			orphanTags = append(orphanTags, tag)
		}
	}

	snapshot.LocalTags = localTags
	snapshot.RemoteTags = remoteTags
	snapshot.NoteMap = noteMap
	snapshot.NoteTags = noteTags
	snapshot.LocalOnlyTags = uniqueSortedStrings(localOnlyTags)
	snapshot.OrphanNotes = uniqueSortedStrings(orphanNotes)
	snapshot.OrphanTags = uniqueSortedStrings(orphanTags)

	return snapshot, nil
}

func printReleaseStateSummary(snapshot releaseStateSnapshot) {
	fmt.Println("Release state summary")
	fmt.Printf("- Local semver tags: %d\n", len(snapshot.LocalTags))
	fmt.Printf("- Remote semver tags: %d\n", len(snapshot.RemoteTags))
	fmt.Printf("- Release notes files: %d\n", len(snapshot.NoteTags))
	fmt.Printf("- Local-only tags (not on remote): %d\n", len(snapshot.LocalOnlyTags))
	fmt.Printf("- Orphan notes (no local tag): %d\n", len(snapshot.OrphanNotes))
	fmt.Printf("- Orphan tags (no release note): %d\n", len(snapshot.OrphanTags))

	if len(snapshot.LocalOnlyTags) > 0 {
		fmt.Println("Local-only tags:")
		for _, tag := range snapshot.LocalOnlyTags {
			fmt.Printf("  %s\n", tag)
		}
	}

	if len(snapshot.OrphanNotes) > 0 {
		fmt.Println("Orphan release notes:")
		for _, tag := range snapshot.OrphanNotes {
			fmt.Printf("  RELEASE_NOTES_%s.md\n", tag)
		}
	}

	if len(snapshot.OrphanTags) > 0 {
		fmt.Println("Orphan tags:")
		for _, tag := range snapshot.OrphanTags {
			fmt.Printf("  %s\n", tag)
		}
	}
}

func getLocalSemverTags(repoRoot string) ([]string, error) {
	output, err := runCommandCapture(repoRoot, "git", []string{"tag", "--list", "v*"})
	if err != nil {
		return nil, errors.New("failed to read local tags")
	}

	lines := strings.Split(strings.ReplaceAll(output, "\r\n", "\n"), "\n")
	tags := make([]string, 0, len(lines))
	for _, line := range lines {
		tag := strings.TrimSpace(line)
		if tag == "" {
			continue
		}
		if _, ok := parseSemverTagInfo(tag); !ok {
			continue
		}
		tags = append(tags, tag)
	}

	return uniqueSortedStrings(tags), nil
}

func getRemoteSemverTagsForSync(repoRoot string) []string {
	if _, err := runCommandCapture(repoRoot, "git", []string{"fetch", "--tags", "--prune", "--quiet"}); err != nil {
		fmt.Fprintln(os.Stderr, "Warning: failed to fetch remote tags, skip remote sync check.")
		return []string{}
	}

	output, err := runCommandCapture(repoRoot, "git", []string{"ls-remote", "--tags", "origin", "v*"})
	if err != nil {
		fmt.Fprintln(os.Stderr, "Warning: failed to list remote tags, skip remote sync check.")
		return []string{}
	}

	return parseRemoteSemverTags(output)
}

func getRemoteSemverTagsForIndex(repoRoot string, skipRemote bool) ([]string, bool) {
	if skipRemote {
		return []string{}, false
	}

	if _, err := runCommandCapture(repoRoot, "git", []string{"fetch", "--tags", "--prune", "--quiet"}); err != nil {
		return []string{}, false
	}

	output, err := runCommandCapture(repoRoot, "git", []string{"ls-remote", "--tags", "origin", "v*"})
	if err != nil {
		return []string{}, false
	}

	return parseRemoteSemverTags(output), true
}

func parseRemoteSemverTags(output string) []string {
	lines := strings.Split(strings.ReplaceAll(output, "\r\n", "\n"), "\n")
	tags := make([]string, 0, len(lines))
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		parts := strings.Split(line, "\t")
		if len(parts) < 2 {
			continue
		}

		ref := strings.TrimSpace(parts[1])
		ref = strings.TrimSuffix(ref, "^{}")
		if !strings.HasPrefix(ref, "refs/tags/") {
			continue
		}

		tag := strings.TrimPrefix(ref, "refs/tags/")
		if _, ok := parseSemverTagInfo(tag); ok {
			tags = append(tags, tag)
		}
	}

	return uniqueSortedStrings(tags)
}

func getReleaseNoteMap(releaseNotesDir string) (map[string]string, error) {
	entries, err := os.ReadDir(releaseNotesDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read release notes directory: %w", err)
	}

	noteMap := make(map[string]string)
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}

		name := entry.Name()
		match := regexp.MustCompile(`^RELEASE_NOTES_(v\d+\.\d+\.\d+(?:[-.].+)?)\.md$`).FindStringSubmatch(name)
		if len(match) != 2 {
			continue
		}

		noteMap[match[1]] = filepath.Join(releaseNotesDir, name)
	}

	return noteMap, nil
}

func deleteLocalTag(repoRoot string, tag string) error {
	if _, err := runCommandCapture(repoRoot, "git", []string{"tag", "-d", tag}); err != nil {
		return fmt.Errorf("failed to delete local tag: %s", tag)
	}
	return nil
}

func updateReleaseIndex(repoRoot string, skipRemote bool) error {
	releaseNotesDir := filepath.Join(repoRoot, "release_notes")
	archiveRoot := filepath.Join(releaseNotesDir, "archive_assets")
	indexPath := filepath.Join(releaseNotesDir, "RELEASE_INDEX.md")

	if !fileExists(releaseNotesDir) {
		return fmt.Errorf("release_notes directory not found: %s", releaseNotesDir)
	}

	noteMap, err := getReleaseNoteMap(releaseNotesDir)
	if err != nil {
		return err
	}

	localTags, err := getLocalSemverTags(repoRoot)
	if err != nil {
		return errors.New("failed to read local git tags")
	}

	remoteTags, remoteEnabled := getRemoteSemverTagsForIndex(repoRoot, skipRemote)

	allTags := uniqueSortedStrings(append(mapKeys(noteMap), localTags...))
	rows := make([]releaseIndexRow, 0, len(allTags))
	for _, tag := range allTags {
		tagInfo, ok := parseSemverTagInfo(tag)
		if !ok {
			continue
		}

		archiveStatus, archivePath := getArchiveStatus(archiveRoot, tag)
		releaseNotesPath := "-"
		if fullPath, exists := noteMap[tag]; exists {
			releaseNotesPath = filepath.ToSlash(filepath.Join("release_notes", filepath.Base(fullPath)))
		}

		localTagStatus := "missing"
		if containsString(localTags, tag) {
			localTagStatus = "present"
		}

		remoteTagStatus := "unknown"
		if remoteEnabled {
			if containsString(remoteTags, tag) {
				remoteTagStatus = "present"
			} else {
				remoteTagStatus = "missing"
			}
		}

		rows = append(rows, releaseIndexRow{
			Major:            tagInfo.Major,
			Minor:            tagInfo.Minor,
			Patch:            tagInfo.Patch,
			SuffixRank:       tagInfo.SuffixRank,
			Suffix:           tagInfo.Suffix,
			Version:          tagInfo.Version,
			Tag:              tagInfo.Tag,
			LocalTagStatus:   localTagStatus,
			RemoteTagStatus:  remoteTagStatus,
			ReleaseNotesPath: releaseNotesPath,
			ArchiveStatus:    archiveStatus,
			ArchivePath:      archivePath,
		})
	}

	sort.Slice(rows, func(i, j int) bool {
		if rows[i].Major != rows[j].Major {
			return rows[i].Major > rows[j].Major
		}
		if rows[i].Minor != rows[j].Minor {
			return rows[i].Minor > rows[j].Minor
		}
		if rows[i].Patch != rows[j].Patch {
			return rows[i].Patch > rows[j].Patch
		}
		if rows[i].SuffixRank != rows[j].SuffixRank {
			return rows[i].SuffixRank > rows[j].SuffixRank
		}
		return rows[i].Suffix > rows[j].Suffix
	})

	lines := []string{
		"# Release Index",
		"",
		"此文件由 scripts/go/rusktask update-release-index 生成，用于记录版本、Tag、本地/远端 Tag 状态与归档状态。",
		"",
		"| Version | Tag | Local Tag Status | Remote Tag Status | Release Notes | Local Archive Status | Local Archive Path |",
		"|---|---|---|---|---|---|---|",
	}

	if len(rows) == 0 {
		lines = append(lines, "| - | - | - | - | - | - | - |")
	} else {
		for _, row := range rows {
			lines = append(lines, fmt.Sprintf("| %s | %s | %s | %s | %s | %s | %s |",
				row.Version,
				row.Tag,
				row.LocalTagStatus,
				row.RemoteTagStatus,
				row.ReleaseNotesPath,
				row.ArchiveStatus,
				row.ArchivePath,
			))
		}
	}

	lines = append(lines, "", fmt.Sprintf("更新时间(UTC): %s", time.Now().UTC().Format("2006-01-02 15:04:05")))
	content := strings.Join(lines, "\n")

	if err := os.WriteFile(indexPath, []byte(content), 0o644); err != nil {
		return fmt.Errorf("failed to write release index: %w", err)
	}

	fmt.Printf("Updated release index: %s\n", indexPath)
	return nil
}

func parseSemverTagInfo(tag string) (semverTagInfo, bool) {
	rx := regexp.MustCompile(`^v(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?P<suffix>[-.].+)?$`)
	match := rx.FindStringSubmatch(tag)
	if len(match) == 0 {
		return semverTagInfo{}, false
	}

	major := 0
	minor := 0
	patch := 0
	_, _ = fmt.Sscanf(match[1], "%d", &major)
	_, _ = fmt.Sscanf(match[2], "%d", &minor)
	_, _ = fmt.Sscanf(match[3], "%d", &patch)

	suffix := ""
	if len(match) > 4 {
		suffix = match[4]
	}

	suffixRank := 0
	if suffix == "" {
		suffixRank = 1
	}

	return semverTagInfo{
		Tag:        tag,
		Version:    strings.TrimPrefix(tag, "v"),
		Major:      major,
		Minor:      minor,
		Patch:      patch,
		Suffix:     suffix,
		SuffixRank: suffixRank,
	}, true
}

func getArchiveStatus(archiveRoot string, tag string) (string, string) {
	archiveDir := filepath.Join(archiveRoot, tag)
	if !fileExists(archiveDir) {
		return "not-archived", "-"
	}

	hasFiles, err := hasAnyRegularFile(archiveDir)
	if err != nil {
		return "empty", filepath.ToSlash(filepath.Join("release_notes", "archive_assets", tag))
	}

	if hasFiles {
		return "archived", filepath.ToSlash(filepath.Join("release_notes", "archive_assets", tag))
	}

	return "empty", filepath.ToSlash(filepath.Join("release_notes", "archive_assets", tag))
}

func hasAnyRegularFile(root string) (bool, error) {
	found := false
	err := filepath.WalkDir(root, func(_ string, d os.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		if d.IsDir() {
			return nil
		}
		found = true
		return errStopWalk
	})

	if err != nil && !errors.Is(err, errStopWalk) {
		return false, err
	}

	return found, nil
}

func containsString(values []string, target string) bool {
	for _, value := range values {
		if value == target {
			return true
		}
	}
	return false
}

func mapKeys(m map[string]string) []string {
	keys := make([]string, 0, len(m))
	for key := range m {
		keys = append(keys, key)
	}
	return keys
}

func runWorkspaceCleanup(args []string) int {
	fs := flag.NewFlagSet("workspace-cleanup", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	mode := fs.String("mode", "apply", "Mode: audit or apply")
	strict := fs.Bool("strict", false, "Fail with exit code 2 when transient paths remain")
	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if *mode != "audit" && *mode != "apply" {
		fmt.Fprintln(os.Stderr, "--mode must be audit or apply")
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	config, err := loadGovernanceConfig(repoRoot)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	found, err := getCleanupCandidates(repoRoot, config.Cleanup)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	fmt.Println("Process file cleanup summary")
	fmt.Printf("- Mode: %s\n", *mode)
	fmt.Printf("- Candidates found: %d\n", len(found))
	for _, path := range found {
		fmt.Printf("  %s\n", path)
	}

	if *mode == "apply" {
		repoRootAbs, err := filepath.Abs(repoRoot)
		if err != nil {
			fmt.Fprintf(os.Stderr, "failed to resolve repo root: %v\n", err)
			return exitExecution
		}

		protectedAbs := make([]string, 0, len(config.Cleanup.ProtectedRelativePaths))
		for _, rel := range config.Cleanup.ProtectedRelativePaths {
			pathAbs, err := filepath.Abs(filepath.Join(repoRoot, filepath.FromSlash(rel)))
			if err != nil {
				fmt.Fprintf(os.Stderr, "failed to resolve protected path %q: %v\n", rel, err)
				return exitExecution
			}
			protectedAbs = append(protectedAbs, pathAbs)
		}

		for _, path := range found {
			resolvedPath, err := filepath.Abs(path)
			if err != nil {
				fmt.Fprintf(os.Stderr, "failed to resolve path %q: %v\n", path, err)
				return exitExecution
			}

			if samePath(resolvedPath, repoRootAbs) {
				fmt.Fprintf(os.Stderr, "refuse to delete repo root: %s\n", resolvedPath)
				return exitExecution
			}
			if isProtectedPath(resolvedPath, protectedAbs) {
				fmt.Fprintf(os.Stderr, "refuse to delete protected path: %s\n", resolvedPath)
				return exitExecution
			}

			if err := os.RemoveAll(resolvedPath); err != nil {
				fmt.Fprintf(os.Stderr, "failed to remove %s: %v\n", resolvedPath, err)
				return exitExecution
			}
			fmt.Printf("Removed: %s\n", resolvedPath)
		}
	}

	remaining := found
	if *mode == "apply" {
		remaining, err = getCleanupCandidates(repoRoot, config.Cleanup)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}
	}

	if *strict && len(remaining) > 0 {
		return exitUsage
	}

	fmt.Printf("cleanup-process-files %s completed.\n", *mode)

	return exitSuccess
}

func runWorkspaceGuard(args []string) int {
	fs := flag.NewFlagSet("workspace-guard", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	mode := fs.String("mode", "audit", "Mode: audit or apply")
	strict := fs.Bool("strict", true, "Fail with exit code 2 when violations are found")
	useStagedPaths := fs.Bool("use-staged-paths", false, "Also validate staged paths against blocked rules")
	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if *mode != "audit" && *mode != "apply" {
		fmt.Fprintln(os.Stderr, "--mode must be audit or apply")
		return exitUsage
	}

	repoRoot, code := requireRepoRoot()
	if code != exitSuccess {
		return code
	}

	config, err := loadGovernanceConfig(repoRoot)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	if len(config.Workspace.AllowedRootEntries) == 0 || len(config.Workspace.BlockedPathRegex) == 0 {
		fmt.Fprintln(os.Stderr, "workspace-governance workspace policy is incomplete")
		return exitExecution
	}

	allowedRootRegexes, err := compileRegexList(config.Workspace.AllowedRootRegex)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid allowedRootRegex: %v\n", err)
		return exitExecution
	}

	blockedPathRegexes, err := compileRegexList(config.Workspace.BlockedPathRegex)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid blockedPathRegex: %v\n", err)
		return exitExecution
	}

	entries, err := os.ReadDir(repoRoot)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to list repo root: %v\n", err)
		return exitExecution
	}

	unexpectedRootEntries := make([]string, 0)
	for _, entry := range entries {
		if !isAllowedRoot(entry.Name(), config.Workspace.AllowedRootEntries, allowedRootRegexes) {
			unexpectedRootEntries = append(unexpectedRootEntries, entry.Name())
		}
	}
	unexpectedRootEntries = uniqueSortedStrings(unexpectedRootEntries)

	blockedWorkspacePaths, err := getBlockedWorkspacePaths(repoRoot, config.Workspace)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return exitExecution
	}

	blockedStagedPaths := make([]string, 0)
	if *useStagedPaths {
		stagedPaths, err := getStagedPaths(repoRoot)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}

		for _, rel := range stagedPaths {
			if matchesAnyRegex(rel, blockedPathRegexes) {
				blockedStagedPaths = append(blockedStagedPaths, rel)
			}
		}
		blockedStagedPaths = uniqueSortedStrings(blockedStagedPaths)
	}

	fmt.Println("Workspace structure summary")
	fmt.Printf("- Mode: %s\n", *mode)
	fmt.Printf("- Unexpected root entries: %d\n", len(unexpectedRootEntries))
	fmt.Printf("- Blocked workspace paths: %d\n", len(blockedWorkspacePaths))
	fmt.Printf("- Blocked staged paths: %d\n", len(blockedStagedPaths))

	if len(unexpectedRootEntries) > 0 {
		fmt.Println("Unexpected root entries:")
		for _, entry := range unexpectedRootEntries {
			fmt.Printf("  %s\n", entry)
		}
	}

	if len(blockedWorkspacePaths) > 0 {
		fmt.Println("Blocked workspace paths:")
		for _, path := range blockedWorkspacePaths {
			fmt.Printf("  %s\n", path)
		}
	}

	if len(blockedStagedPaths) > 0 {
		fmt.Println("Blocked staged paths:")
		for _, path := range blockedStagedPaths {
			fmt.Printf("  %s\n", path)
		}
	}

	if *mode == "apply" {
		cleanupCode := runWorkspaceCleanup([]string{"--mode", "apply"})
		if cleanupCode != exitSuccess {
			return cleanupCode
		}

		blockedWorkspacePaths, err = getBlockedWorkspacePaths(repoRoot, config.Workspace)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%v\n", err)
			return exitExecution
		}
	}

	issueCount := len(unexpectedRootEntries) + len(blockedWorkspacePaths) + len(blockedStagedPaths)
	if *strict && issueCount > 0 {
		return exitUsage
	}

	fmt.Printf("enforce-workspace-structure %s completed.\n", *mode)

	return exitSuccess
}

func loadGovernanceConfig(repoRoot string) (governanceConfig, error) {
	config := governanceConfig{}
	configPath := filepath.Join(repoRoot, filepath.FromSlash("scripts/workspace-governance.json"))

	content, err := os.ReadFile(configPath)
	if err != nil {
		return config, fmt.Errorf("missing workspace governance config: %s", configPath)
	}

	if err := json.Unmarshal(content, &config); err != nil {
		return config, fmt.Errorf("failed to parse workspace governance config: %w", err)
	}

	return config, nil
}

func getCleanupCandidates(repoRoot string, cleanup cleanupPolicy) ([]string, error) {
	if len(cleanup.FixedRelativePaths) == 0 {
		return nil, errors.New("workspace-governance cleanup.fixedRelativePaths cannot be empty")
	}

	candidates := make([]string, 0)
	for _, rel := range cleanup.FixedRelativePaths {
		fullPath := filepath.Join(repoRoot, filepath.FromSlash(rel))
		if !fileExists(fullPath) {
			continue
		}

		resolved, err := filepath.Abs(fullPath)
		if err != nil {
			return nil, fmt.Errorf("failed to resolve cleanup path %q: %w", rel, err)
		}
		candidates = append(candidates, resolved)
	}

	for _, pattern := range cleanup.GlobPatterns {
		globPattern := filepath.Join(repoRoot, filepath.FromSlash(pattern))
		matches, err := filepath.Glob(globPattern)
		if err != nil {
			return nil, fmt.Errorf("invalid cleanup glob pattern %q: %w", pattern, err)
		}

		for _, match := range matches {
			if !fileExists(match) {
				continue
			}

			resolved, err := filepath.Abs(match)
			if err != nil {
				return nil, fmt.Errorf("failed to resolve cleanup match %q: %w", match, err)
			}
			candidates = append(candidates, resolved)
		}
	}

	return uniqueSortedStrings(candidates), nil
}

func getBlockedWorkspacePaths(repoRoot string, workspace workspacePolicy) ([]string, error) {
	paths := make([]string, 0)

	for _, rel := range workspace.BlockedFixedRelativePaths {
		fullPath := filepath.Join(repoRoot, filepath.FromSlash(rel))
		if fileExists(fullPath) {
			paths = append(paths, normalizePath(rel))
		}
	}

	for _, pattern := range workspace.BlockedGlobPatterns {
		globPattern := filepath.Join(repoRoot, filepath.FromSlash(pattern))
		matches, err := filepath.Glob(globPattern)
		if err != nil {
			return nil, fmt.Errorf("invalid blocked glob pattern %q: %w", pattern, err)
		}

		for _, match := range matches {
			relPath, err := filepath.Rel(repoRoot, match)
			if err != nil {
				continue
			}
			paths = append(paths, normalizePath(relPath))
		}
	}

	return uniqueSortedStrings(paths), nil
}

func getStagedPaths(repoRoot string) ([]string, error) {
	cmd := exec.Command("git", "diff", "--cached", "--name-only")
	cmd.Dir = repoRoot
	out, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("failed to read staged paths: %w", err)
	}

	lines := strings.Split(strings.ReplaceAll(string(out), "\r\n", "\n"), "\n")
	paths := make([]string, 0, len(lines))
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}
		paths = append(paths, normalizePath(trimmed))
	}

	return uniqueSortedStrings(paths), nil
}

func compileRegexList(patterns []string) ([]*regexp.Regexp, error) {
	compiled := make([]*regexp.Regexp, 0, len(patterns))
	for _, pattern := range patterns {
		rx, err := regexp.Compile(pattern)
		if err != nil {
			return nil, fmt.Errorf("%s: %w", pattern, err)
		}
		compiled = append(compiled, rx)
	}
	return compiled, nil
}

func isAllowedRoot(entry string, allowedEntries []string, allowedRegexes []*regexp.Regexp) bool {
	for _, allowed := range allowedEntries {
		if entry == allowed {
			return true
		}
	}

	for _, rx := range allowedRegexes {
		if rx.MatchString(entry) {
			return true
		}
	}

	return false
}

func matchesAnyRegex(value string, regexes []*regexp.Regexp) bool {
	for _, rx := range regexes {
		if rx.MatchString(value) {
			return true
		}
	}
	return false
}

func normalizePath(path string) string {
	normalized := filepath.Clean(path)
	normalized = strings.ReplaceAll(normalized, "\\", "/")
	normalized = strings.TrimPrefix(normalized, "./")
	normalized = strings.Trim(normalized, "/")
	return normalized
}

func uniqueSortedStrings(values []string) []string {
	if len(values) == 0 {
		return []string{}
	}

	set := make(map[string]struct{}, len(values))
	for _, value := range values {
		trimmed := strings.TrimSpace(value)
		if trimmed == "" {
			continue
		}
		set[trimmed] = struct{}{}
	}

	result := make([]string, 0, len(set))
	for value := range set {
		result = append(result, value)
	}
	sort.Strings(result)
	return result
}

func samePath(a string, b string) bool {
	cleanA := filepath.Clean(a)
	cleanB := filepath.Clean(b)
	if os.PathSeparator == '\\' {
		return strings.EqualFold(cleanA, cleanB)
	}
	return cleanA == cleanB
}

func isProtectedPath(candidate string, protectedAbs []string) bool {
	cleanCandidate := filepath.Clean(candidate)
	for _, protected := range protectedAbs {
		cleanProtected := filepath.Clean(protected)
		if samePath(cleanCandidate, cleanProtected) {
			return true
		}

		prefix := cleanProtected + string(os.PathSeparator)
		if os.PathSeparator == '\\' {
			if strings.HasPrefix(strings.ToLower(cleanCandidate), strings.ToLower(prefix)) {
				return true
			}
		} else if strings.HasPrefix(cleanCandidate, prefix) {
			return true
		}
	}

	return false
}

func runCommand(workingDir string, name string, args []string, env map[string]string) error {
	fmt.Printf("[INFO] running: %s %s\n", name, strings.Join(args, " "))

	cmd := exec.Command(name, args...)
	cmd.Dir = workingDir
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if len(env) > 0 {
		cmd.Env = os.Environ()
		for key, value := range env {
			cmd.Env = append(cmd.Env, fmt.Sprintf("%s=%s", key, value))
		}
	}

	if err := cmd.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "[ERROR] command failed: %v\n", err)
		return err
	}

	return nil
}

func runCommandCapture(workingDir string, name string, args []string) (string, error) {
	cmd := exec.Command(name, args...)
	cmd.Dir = workingDir
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("command failed: %s %s (%w): %s", name, strings.Join(args, " "), err, strings.TrimSpace(string(output)))
	}

	return string(output), nil
}

func requireRepoRoot() (string, int) {
	repoRoot, err := detectRepoRoot()
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		return "", exitPrecondition
	}
	return repoRoot, exitSuccess
}

func detectRepoRoot() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", fmt.Errorf("failed to get cwd: %w", err)
	}

	current := cwd
	for {
		gitDir := filepath.Join(current, ".git")
		goMod := filepath.Join(current, filepath.FromSlash("scripts/go/rusktask/go.mod"))
		if fileExists(gitDir) && fileExists(goMod) {
			return current, nil
		}

		parent := filepath.Dir(current)
		if parent == current {
			break
		}
		current = parent
	}

	return "", errors.New("unable to locate repository root (expected .git and scripts/go/rusktask/go.mod)")
}

func fileExists(path string) bool {
	if _, err := os.Stat(path); err != nil {
		return false
	}
	return true
}

func runReleaseNotes(args []string) int {
	if len(args) == 0 {
		fmt.Fprintln(os.Stderr, "missing subcommand: release-notes validate")
		return exitUsage
	}

	switch args[0] {
	case "validate":
		return runReleaseNotesValidate(args[1:])
	default:
		fmt.Fprintf(os.Stderr, "unknown release-notes subcommand: %s\n", args[0])
		return exitUsage
	}
}

func runReleaseNotesValidate(args []string) int {
	fs := flag.NewFlagSet("release-notes validate", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)

	file := fs.String("file", "", "Path to release notes markdown file")
	mode := fs.String("mode", "release", "Validation mode: draft or release")

	if err := fs.Parse(args); err != nil {
		return exitUsage
	}

	if *file == "" {
		fmt.Fprintln(os.Stderr, "--file is required")
		return exitUsage
	}

	if *mode != "draft" && *mode != "release" {
		fmt.Fprintln(os.Stderr, "--mode must be draft or release")
		return exitUsage
	}

	contentBytes, err := os.ReadFile(*file)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to read release notes: %v\n", err)
		return exitExecution
	}

	if err := validateReleaseNotes(string(contentBytes), *mode); err != nil {
		fmt.Fprintf(os.Stderr, "release notes validation failed: %v\n", err)
		return exitExecution
	}

	fmt.Printf("release notes validation passed (%s): %s\n", *mode, *file)
	return exitSuccess
}

func validateReleaseNotes(content string, mode string) error {
	required := []struct {
		pattern string
		err     string
	}{
		{`(?m)^#\s+v\d+\.\d+\.\d+([-.][A-Za-z0-9.-]+)?\s*$`, "missing release title header '# vX.Y.Z'"},
		{`(?m)^##\s+Highlights\s*$`, "missing '## Highlights' section"},
		{`(?m)^##\s+Fixes\s*$`, "missing '## Fixes' section"},
		{`(?m)^##\s+Verification\s*$`, "missing '## Verification' section"},
		{`(?m)^-\s+\[[ xX]\]\s+(?:(?:\./|\.\\)?scripts[/\\]task(?:\.ps1)?|make\.ps1)\s+preflight\s*$`, "verification list must include 'scripts/task preflight'"},
		{`(?m)^-\s+\[[ xX]\]\s+CI\s+passed\s*$`, "verification list must include 'CI passed'"},
		{`(?m)^-\s+\[[ xX]\]\s+Release\s+assets\s+verified\s+\(exe/setup/checksums\)\s*$`, "verification list must include release asset verification item"},
	}

	for _, item := range required {
		rx := regexp.MustCompile(item.pattern)
		if !rx.MatchString(content) {
			return errors.New(item.err)
		}
	}

	if mode == "release" {
		placeholderMarkers := []string{
			"Describe major improvements here.",
			"Describe bug fixes here.",
		}

		for _, marker := range placeholderMarkers {
			if strings.Contains(content, marker) {
				return fmt.Errorf("placeholder text still present: %q", marker)
			}
		}

		strictChecks := []struct {
			pattern string
			err     string
		}{
			{`(?m)^-\s+\[[xX]\]\s+(?:(?:\./|\.\\)?scripts[/\\]task(?:\.ps1)?|make\.ps1)\s+preflight\s*$`, "release mode requires checked item: scripts/task preflight"},
			{`(?m)^-\s+\[[xX]\]\s+CI\s+passed\s*$`, "release mode requires checked item: CI passed"},
			{`(?m)^-\s+\[[xX]\]\s+Release\s+assets\s+verified\s+\(exe/setup/checksums\)\s*$`, "release mode requires checked item: Release assets verified"},
		}

		for _, item := range strictChecks {
			rx := regexp.MustCompile(item.pattern)
			if !rx.MatchString(content) {
				return errors.New(item.err)
			}
		}

		highlights := sectionBullets(content, "Highlights")
		fixes := sectionBullets(content, "Fixes")
		if meaningfulBulletCount(highlights) < 1 {
			return errors.New("release mode requires at least one meaningful bullet in Highlights")
		}
		if meaningfulBulletCount(fixes) < 1 {
			return errors.New("release mode requires at least one meaningful bullet in Fixes")
		}
	}

	return nil
}

func sectionBullets(content string, section string) []string {
	rx := regexp.MustCompile(`(?ms)^##\s+` + regexp.QuoteMeta(section) + `\s*$\r?\n(.*?)(?=^##\s|\z)`)
	m := rx.FindStringSubmatch(content)
	if len(m) < 2 {
		return nil
	}

	lines := strings.Split(m[1], "\n")
	bullets := make([]string, 0, len(lines))
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "- ") {
			bullets = append(bullets, trimmed)
		}
	}
	return bullets
}

func meaningfulBulletCount(bullets []string) int {
	placeholders := map[string]struct{}{
		"describe major improvements here": {},
		"describe bug fixes here":          {},
	}

	count := 0
	for _, bullet := range bullets {
		text := strings.TrimSpace(strings.TrimPrefix(bullet, "- "))
		textLower := strings.ToLower(text)
		if textLower == "" {
			continue
		}
		if _, exists := placeholders[textLower]; exists {
			continue
		}
		count++
	}

	return count
}

func printUsage() {
	fmt.Println("rusktask - transitional Go task orchestrator")
	fmt.Println("")
	fmt.Println("Usage:")
	fmt.Println("  rusktask <command> [arguments]")
	fmt.Println("")
	fmt.Println("Commands:")
	fmt.Println("  fmt [--check]")
	fmt.Println("  clippy")
	fmt.Println("  test [--release]")
	fmt.Println("  build [--release]")
	fmt.Println("  doc")
	fmt.Println("  audit [--ignore <ID>]...")
	fmt.Println("  check")
	fmt.Println("  preflight")
	fmt.Println("  git-check [--pre-push] [--commit-msg-file <path>]")
	fmt.Println("  rust-review [--fix] [--skip-tests] [--skip-audit] [--project <name>]...")
	fmt.Println("  review [--quick] [--fix] [--before-push] [--skip-tests] [--skip-audit] [--project <name>]...")
	fmt.Println("  install-hooks [--uninstall] [--force]")
	fmt.Println("  smart-bump [--part patch|minor|major] [--push] [--no-verify] [--allow-dirty] [--no-tag] [--skip-release-state-audit] [--skip-process-cleanup] [--skip-workspace-guard]")
	fmt.Println("  smart-rollback --tag <vX.Y.Z> [--owner <owner>] [--repo <repo>] [--delete-release] [--delete-remote-tag] [--delete-local-tag] [--revert-last-commit] [--push-revert] [--no-verify] [--skip-process-cleanup] [--skip-workspace-guard] [--skip-index-refresh]")
	fmt.Println("  pr-helper [--check|--create|--merge] [--draft] [--title <text>] [--body <text>] [--base <branch>] [--head <branch>] [--auto-fill]")
	fmt.Println("  release-sync [--mode audit|apply] [--prune-local-tags-not-on-remote] [--clean-orphan-notes] [--skip-remote] [--strict]")
	fmt.Println("  workflow-seal [--mode audit|apply] [--prune-local-tags-not-on-remote] [--clean-orphan-notes] [--skip-remote]")
	fmt.Println("  workspace-cleanup [--mode audit|apply] [--strict]")
	fmt.Println("  workspace-guard [--mode audit|apply] [--strict] [--use-staged-paths]")
	fmt.Println("  docs-bundle [--output-root <dir>] [--create-zip]")
	fmt.Println("  release-publish [--owner <owner>] [--repo <repo>] --tag <tag> [--release-name <name>] [--body-file <path>] [--asset <path>]... [--prerelease] [--draft] [--prune-extra-assets]")
	fmt.Println("  build-release-slim [--skip-tests] [--skip-clippy]")
	fmt.Println("  package-windows-installer [--version <X.Y.Z>] [--build-tag <yyyymmdd>] [--prefer-iexpress] [--skip-build]")
	fmt.Println("  package-windows-assets [--version <X.Y.Z>] [--output-dir <dir>] [--skip-build]")
	fmt.Println("  package-windows-portable-installer [--version <X.Y.Z>] [--output-dir <dir>] [--skip-build]")
	fmt.Println("  update-release-index [--skip-remote]")
	fmt.Println("  version")
	fmt.Println("  release-notes validate --file <path> --mode <draft|release>")
}
