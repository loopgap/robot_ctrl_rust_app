# ============================================================
# rust_serial Workspace — 统一构建自动化
# ============================================================
# 用法:  ./scripts/ubuntu/task.sh <target>（或 make -f scripts/ubuntu/Makefile <target>）
#   ./scripts/ubuntu/task.sh check      — 格式 + clippy + 测试（快速校验）
#   ./scripts/ubuntu/task.sh build      — Debug 构建正式发布项目
#   ./scripts/ubuntu/task.sh release    — Release 构建正式发布项目
#   ./scripts/ubuntu/task.sh clean      — 清理所有 target/
#   ./scripts/ubuntu/task.sh fmt        — 自动格式化
#   ./scripts/ubuntu/task.sh test       — 运行全部测试
#   ./scripts/ubuntu/task.sh audit      — 安全审计
#   ./scripts/ubuntu/task.sh doc        — 生成文档
#   ./scripts/ubuntu/task.sh preflight  — 发布前完整校验
# ============================================================

SHELL := /bin/bash
AUDIT_DB := $(CURDIR)/.cargo-advisory-db
AUDIT_IGNORE_IDS := RUSTSEC-2023-0071
AUDIT_IGNORE_FLAGS := $(foreach id,$(AUDIT_IGNORE_IDS),--ignore $(id))
RUSKTASK := cd scripts/go/rusktask && go run .
SELF_MAKE := $(MAKE) -f $(abspath $(lastword $(MAKEFILE_LIST)))

ROBOT_CONTROL_PROJECT := $(if $(wildcard crates/robot_control/Cargo.toml),crates/robot_control,robot_control_rust)
TOOLS_SUITE_PROJECT := $(if $(wildcard crates/tools_suite/Cargo.toml),crates/tools_suite,rust_tools_suite)
CORE_PROJECTS = $(ROBOT_CONTROL_PROJECT) $(TOOLS_SUITE_PROJECT)
ALL_PROJECTS := $(CORE_PROJECTS) $(if $(wildcard crates/robot_core/Cargo.toml),crates/robot_core,) $(if $(wildcard crates/devtools/Cargo.toml),crates/devtools,)
AUDIT_LOCK_FILES := $(strip $(if $(wildcard Cargo.lock),Cargo.lock,) $(if $(wildcard $(ROBOT_CONTROL_PROJECT)/Cargo.lock),$(ROBOT_CONTROL_PROJECT)/Cargo.lock,) $(if $(wildcard $(TOOLS_SUITE_PROJECT)/Cargo.lock),$(TOOLS_SUITE_PROJECT)/Cargo.lock,))
DENY_MANIFESTS := $(strip $(if $(wildcard Cargo.toml),Cargo.toml,) $(if $(wildcard $(ROBOT_CONTROL_PROJECT)/Cargo.toml),$(ROBOT_CONTROL_PROJECT)/Cargo.toml,) $(if $(wildcard $(TOOLS_SUITE_PROJECT)/Cargo.toml),$(TOOLS_SUITE_PROJECT)/Cargo.toml,))
RELEASE_NOTES_FILE ?=
RELEASE_NOTES_MODE ?= release
PACKAGE_VERSION ?=
PACKAGE_BUILD_TAG ?=
PACKAGE_OUTPUT_DIR ?=
PACKAGE_SKIP_BUILD ?= 0
DOCS_OUTPUT_ROOT ?=
DOCS_CREATE_ZIP ?= 0
RELEASE_OWNER ?= loopgap
RELEASE_REPO ?= robot_ctrl_rust_app
RELEASE_TAG ?=
RELEASE_NAME ?=
RELEASE_BODY_FILE ?=
RELEASE_ASSET ?=
RELEASE_PRERELEASE ?= 0
RELEASE_DRAFT ?= 0
RELEASE_PRUNE_EXTRA_ASSETS ?= 0
BUMP_PART ?= patch
BUMP_PUSH ?= 0
BUMP_NO_VERIFY ?= 0
BUMP_ALLOW_DIRTY ?= 0
BUMP_NO_TAG ?= 0
BUMP_SKIP_RELEASE_STATE_AUDIT ?= 0
BUMP_SKIP_PROCESS_CLEANUP ?= 0
BUMP_SKIP_WORKSPACE_GUARD ?= 0
ROLLBACK_TAG ?=
ROLLBACK_OWNER ?= loopgap
ROLLBACK_REPO ?= robot_ctrl_rust_app
ROLLBACK_DELETE_RELEASE ?= 0
ROLLBACK_DELETE_REMOTE_TAG ?= 0
ROLLBACK_DELETE_LOCAL_TAG ?= 0
ROLLBACK_REVERT_LAST_COMMIT ?= 0
ROLLBACK_PUSH_REVERT ?= 0
ROLLBACK_NO_VERIFY ?= 0
ROLLBACK_SKIP_PROCESS_CLEANUP ?= 0
ROLLBACK_SKIP_WORKSPACE_GUARD ?= 0
ROLLBACK_SKIP_INDEX_REFRESH ?= 0
PR_CREATE ?= 0
PR_CHECK ?= 0
PR_MERGE ?= 0
PR_DRAFT ?= 0
PR_TITLE ?=
PR_BODY ?=
PR_BASE ?= main
PR_HEAD ?=
PR_AUTO_FILL ?= 0

.PHONY: all check fmt fmt-check clippy test test-release build release clean doc audit preflight ci-local ci-local-full release-sync release-sync-apply release-index workflow-seal workflow-seal-apply workspace-guard workspace-cleanup release-notes-validate smart-bump smart-rollback pr-helper git-check git-review-before-push git-hooks-install git-hooks-uninstall git-pr-check git-pr-create git-pr-merge docs-bundle release-publish build-release-slim package-windows-installer package-windows-assets package-windows-portable-installer release-local-windows release-local-linux release-local-dual go-check go-preflight go-build go-test go-doc go-audit go-release-sync go-workflow-seal go-git-check go-rust-review go-review go-install-hooks help

all: workspace-cleanup workspace-guard check build

fmt:
	@echo "══ 格式化代码 ══"
	@for p in $(ALL_PROJECTS); do \
		echo "→ $$p"; \
		cargo fmt --manifest-path $${p}/Cargo.toml || exit 1; \
	done

fmt-check:
	@echo "== Format check =="
	@for p in $(ALL_PROJECTS); do \
		echo "-> $$p"; \
		if ! cargo fmt --check --manifest-path $${p}/Cargo.toml; then \
			echo ""; \
			echo "Problem: $$p has formatting issues"; \
			echo "Suggested command: cargo fmt --manifest-path $${p}/Cargo.toml"; \
			echo "Fix direction: format first, then rerun ./scripts/ubuntu/task.sh fmt-check"; \
			echo "Inspect first: recently changed .rs files in $$p"; \
			exit 1; \
		fi; \
	done

clippy:
	@echo "== Clippy lint =="
	@for p in $(ALL_PROJECTS); do \
		echo "-> $$p"; \
		if ! cargo clippy --manifest-path $${p}/Cargo.toml --all-targets -- -D warnings; then \
			echo ""; \
			echo "Problem: $$p clippy check failed"; \
			echo "Suggested command: cargo clippy --manifest-path $${p}/Cargo.toml --all-targets -- -D warnings"; \
			echo "Fix direction: resolve the first -D warnings violation"; \
			echo "Inspect first: first error or warning in clippy output"; \
			exit 1; \
		fi; \
	done

test:
	@echo "== Run tests =="
	@for p in $(ALL_PROJECTS); do \
		echo "-> $$p"; \
		if ! cargo test --manifest-path $${p}/Cargo.toml; then \
			echo ""; \
			echo "Problem: $$p tests failed"; \
			echo "Suggested command: cargo test --manifest-path $${p}/Cargo.toml"; \
			echo "Fix direction: reproduce first failing test and isolate root cause"; \
			echo "Inspect first: failing test name and module"; \
			exit 1; \
		fi; \
	done

test-release:
	@echo "== Run release tests =="
	@for p in $(ALL_PROJECTS); do \
		echo "-> $$p"; \
		if ! cargo test --release --manifest-path $${p}/Cargo.toml; then \
			echo ""; \
			echo "Problem: $$p release tests failed"; \
			echo "Suggested command: cargo test --release --manifest-path $${p}/Cargo.toml"; \
			echo "Fix direction: check release-only behavior, feature gates, and init paths"; \
			echo "Inspect first: first failing release test case"; \
			exit 1; \
		fi; \
	done

build:
	@echo "== Debug build =="
	@for p in $(CORE_PROJECTS); do \
		echo "-> $$p"; \
		if ! cargo build --manifest-path $${p}/Cargo.toml; then \
			echo ""; \
			echo "Problem: $$p debug build failed"; \
			echo "Suggested command: cargo build --manifest-path $${p}/Cargo.toml"; \
			echo "Fix direction: resolve the first compiler error, then verify deps/features"; \
			echo "Inspect first: first error in compiler output"; \
			exit 1; \
		fi; \
	done

release:
	@echo "== Release build =="
	@for p in $(CORE_PROJECTS); do \
		echo "-> $$p"; \
		if ! cargo build --release --manifest-path $${p}/Cargo.toml; then \
			echo ""; \
			echo "Problem: $$p release build failed"; \
			echo "Suggested command: cargo build --release --manifest-path $${p}/Cargo.toml"; \
			echo "Fix direction: verify system deps, target config, and release-only branches"; \
			echo "Inspect first: first error in release build output"; \
			exit 1; \
		fi; \
	done
	@echo ""
	@echo "== Build artifacts =="
	@ls -lh target/release/robot_control* 2>/dev/null || true
	@ls -lh target/release/tools_suite* 2>/dev/null || true

doc:
	@echo "== Build docs =="
	@for p in $(ALL_PROJECTS); do \
		echo "-> $$p"; \
		if ! RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --manifest-path $${p}/Cargo.toml; then \
			echo ""; \
			echo "Problem: $$p docs build failed"; \
			echo "Suggested command: RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --manifest-path $${p}/Cargo.toml"; \
			echo "Fix direction: resolve rustdoc warnings and invalid docs comments"; \
			echo "Inspect first: first rustdoc warning/error"; \
			exit 1; \
		fi; \
	done

audit:
	@echo "== Security audit =="
	@if [ -z "$(AUDIT_LOCK_FILES)" ]; then \
		echo ""; \
		echo "Problem: no Cargo.lock file found for security audit"; \
		echo "Suggested command: cargo audit -d $(AUDIT_DB) $(AUDIT_IGNORE_FLAGS) -f <path-to-Cargo.lock>"; \
		echo "Fix direction: ensure at least one project lockfile exists before running audit"; \
		echo "Inspect first: workspace root and core project directories"; \
		exit 1; \
	fi
	@for lock in $(AUDIT_LOCK_FILES); do \
		echo "-> lock: $$lock"; \
		if ! cargo audit -d $(AUDIT_DB) $(AUDIT_IGNORE_FLAGS) -f $$lock; then \
			echo ""; \
			echo "Problem: security audit failed for lockfile $$lock"; \
			echo "Suggested command: cargo audit -d $(AUDIT_DB) $(AUDIT_IGNORE_FLAGS) -f $$lock"; \
			echo "Fix direction: upgrade vulnerable dependencies and regenerate the failing lockfile"; \
			echo "Inspect first: advisory ID and affected crate in audit output"; \
			exit 1; \
		fi; \
	done
	@if [ -z "$(DENY_MANIFESTS)" ]; then \
		echo ""; \
		echo "Problem: no Cargo.toml manifest found for dependency policy check"; \
		echo "Suggested command: cargo deny check advisories bans sources --manifest-path <path> --config $(CURDIR)/deny.toml"; \
		echo "Fix direction: ensure at least one project manifest exists before running cargo-deny"; \
		echo "Inspect first: workspace root and core project directories"; \
		exit 1; \
	fi
	@for manifest in $(DENY_MANIFESTS); do \
		echo "-> manifest: $$manifest"; \
		manifest_dir=$$(dirname "$$manifest"); \
		if ! (cd "$$manifest_dir" && cargo deny check advisories bans sources --config $(CURDIR)/deny.toml); then \
			echo ""; \
			echo "Problem: dependency policy check failed for manifest $$manifest"; \
			echo "Suggested command: cd $$manifest_dir && cargo deny check advisories bans sources --config $(CURDIR)/deny.toml"; \
			echo "Fix direction: review advisories, source policy, and bans rules"; \
			echo "Inspect first: first cargo-deny error"; \
			exit 1; \
		fi; \
	done

clean:
	@echo "══ 清理构建产物 ══"
	@for p in $(ALL_PROJECTS); do \
		cargo clean --manifest-path $${p}/Cargo.toml; \
	done

check: workspace-cleanup workspace-guard fmt-check clippy test
	@echo ""
	@echo "✓ 全部检查通过"

preflight: workspace-cleanup workspace-guard fmt-check clippy test test-release release doc
	@echo ""
	@echo "🚀 Preflight 全部通过，可以发布！"

ci-local:
	@echo "══ 本地 CI（核心）══"
	@$(RUSKTASK) workspace-cleanup --mode audit --strict
	@$(RUSKTASK) workspace-guard --mode audit --strict
	@$(SELF_MAKE) fmt-check
	@$(SELF_MAKE) clippy
	@$(SELF_MAKE) test
	@$(SELF_MAKE) doc

ci-local-full: ci-local
	@echo "══ 本地 CI（完整）══"
	@$(SELF_MAKE) audit

release-sync:
	@echo "══ Release 状态审计 ══"
	@$(RUSKTASK) workspace-cleanup --mode apply
	@$(RUSKTASK) workspace-guard --mode audit --strict
	@$(RUSKTASK) release-sync --mode audit

release-sync-apply:
	@echo "══ Release 状态归一化 ══"
	@$(RUSKTASK) workspace-cleanup --mode apply
	@$(RUSKTASK) release-sync --mode apply --prune-local-tags-not-on-remote --clean-orphan-notes
	@$(RUSKTASK) workspace-cleanup --mode apply
	@$(RUSKTASK) workspace-guard --mode audit --strict

release-index:
	@echo "══ 重建发布索引 ══"
	@$(RUSKTASK) update-release-index

workflow-seal:
	@echo "══ Workflow 固化审计 ══"
	@$(RUSKTASK) workflow-seal --mode audit

workflow-seal-apply:
	@echo "══ Workflow 固化归一化 ══"
	@$(RUSKTASK) workflow-seal --mode apply --prune-local-tags-not-on-remote --clean-orphan-notes

workspace-guard:
	@echo "══ Workspace 结构守卫 ══"
	@$(RUSKTASK) workspace-guard --mode audit --strict

workspace-cleanup:
	@echo "══ Workspace 过程文件清理 ══"
	@$(RUSKTASK) workspace-cleanup --mode apply

release-notes-validate:
	@echo "══ 发布说明校验 ══"
	@if [ -z "$(RELEASE_NOTES_FILE)" ]; then \
		echo "请提供 RELEASE_NOTES_FILE=release_notes/RELEASE_NOTES_vX.Y.Z.md"; \
		exit 2; \
	fi
	@if ! command -v go >/dev/null 2>&1; then \
		echo "go command not found. Please install Go first."; \
		exit 2; \
	fi
	@$(RUSKTASK) release-notes validate --file "$(abspath $(RELEASE_NOTES_FILE))" --mode "$(RELEASE_NOTES_MODE)"

smart-bump:
	@echo "══ 智能版本升级 ══"
	@$(RUSKTASK) smart-bump --part "$(BUMP_PART)" $(if $(filter 1,$(BUMP_PUSH)),--push,) $(if $(filter 1,$(BUMP_NO_VERIFY)),--no-verify,) $(if $(filter 1,$(BUMP_ALLOW_DIRTY)),--allow-dirty,) $(if $(filter 1,$(BUMP_NO_TAG)),--no-tag,) $(if $(filter 1,$(BUMP_SKIP_RELEASE_STATE_AUDIT)),--skip-release-state-audit,) $(if $(filter 1,$(BUMP_SKIP_PROCESS_CLEANUP)),--skip-process-cleanup,) $(if $(filter 1,$(BUMP_SKIP_WORKSPACE_GUARD)),--skip-workspace-guard,)

smart-rollback:
	@echo "══ 智能发布回滚 ══"
	@if [ -z "$(ROLLBACK_TAG)" ]; then \
		echo "请提供 ROLLBACK_TAG=vX.Y.Z"; \
		exit 2; \
	fi
	@$(RUSKTASK) smart-rollback --tag "$(ROLLBACK_TAG)" --owner "$(ROLLBACK_OWNER)" --repo "$(ROLLBACK_REPO)" $(if $(filter 1,$(ROLLBACK_DELETE_RELEASE)),--delete-release,) $(if $(filter 1,$(ROLLBACK_DELETE_REMOTE_TAG)),--delete-remote-tag,) $(if $(filter 1,$(ROLLBACK_DELETE_LOCAL_TAG)),--delete-local-tag,) $(if $(filter 1,$(ROLLBACK_REVERT_LAST_COMMIT)),--revert-last-commit,) $(if $(filter 1,$(ROLLBACK_PUSH_REVERT)),--push-revert,) $(if $(filter 1,$(ROLLBACK_NO_VERIFY)),--no-verify,) $(if $(filter 1,$(ROLLBACK_SKIP_PROCESS_CLEANUP)),--skip-process-cleanup,) $(if $(filter 1,$(ROLLBACK_SKIP_WORKSPACE_GUARD)),--skip-workspace-guard,) $(if $(filter 1,$(ROLLBACK_SKIP_INDEX_REFRESH)),--skip-index-refresh,)

pr-helper:
	@echo "══ PR 辅助工具 ══"
	@$(RUSKTASK) pr-helper --base "$(PR_BASE)" $(if $(filter 1,$(PR_CREATE)),--create,) $(if $(filter 1,$(PR_CHECK)),--check,) $(if $(filter 1,$(PR_MERGE)),--merge,) $(if $(filter 1,$(PR_DRAFT)),--draft,) $(if $(PR_TITLE),--title "$(PR_TITLE)",) $(if $(PR_BODY),--body "$(PR_BODY)",) $(if $(PR_HEAD),--head "$(PR_HEAD)",) $(if $(filter 1,$(PR_AUTO_FILL)),--auto-fill,)

git-check:
	@echo "══ Git 工作流校验 ══"
	@$(RUSKTASK) git-check

git-review-before-push:
	@echo "══ 推送前完整审查 ══"
	@$(RUSKTASK) review --before-push

git-hooks-install:
	@echo "══ 安装 Git Hooks ══"
	@$(RUSKTASK) install-hooks

git-hooks-uninstall:
	@echo "══ 卸载 Git Hooks ══"
	@$(RUSKTASK) install-hooks --uninstall

git-pr-check:
	@echo "══ PR 就绪检查 ══"
	@$(RUSKTASK) pr-helper --check --base "$(PR_BASE)" $(if $(PR_HEAD),--head "$(PR_HEAD)",)

git-pr-create:
	@echo "══ 创建 PR ══"
	@$(RUSKTASK) pr-helper --create --base "$(PR_BASE)" $(if $(filter 1,$(PR_DRAFT)),--draft,) $(if $(PR_TITLE),--title "$(PR_TITLE)",) $(if $(PR_BODY),--body "$(PR_BODY)",) $(if $(PR_HEAD),--head "$(PR_HEAD)",) $(if $(filter 1,$(PR_AUTO_FILL)),--auto-fill,)

git-pr-merge:
	@echo "══ 合并 PR ══"
	@$(RUSKTASK) pr-helper --merge --base "$(PR_BASE)" $(if $(PR_HEAD),--head "$(PR_HEAD)",)

docs-bundle:
	@echo "══ 构建文档打包目录 ══"
	@$(RUSKTASK) docs-bundle $(if $(DOCS_OUTPUT_ROOT),--output-root "$(DOCS_OUTPUT_ROOT)",) $(if $(filter 1,$(DOCS_CREATE_ZIP)),--create-zip,)

release-publish:
	@echo "══ 发布 GitHub Release ══"
	@if [ -z "$(RELEASE_TAG)" ]; then \
		echo "请提供 RELEASE_TAG=vX.Y.Z"; \
		exit 2; \
	fi
	@$(RUSKTASK) release-publish --owner "$(RELEASE_OWNER)" --repo "$(RELEASE_REPO)" --tag "$(RELEASE_TAG)" $(if $(RELEASE_NAME),--release-name "$(RELEASE_NAME)",) $(if $(RELEASE_BODY_FILE),--body-file "$(RELEASE_BODY_FILE)",) $(if $(RELEASE_ASSET),--asset "$(RELEASE_ASSET)",) $(if $(filter 1,$(RELEASE_PRERELEASE)),--prerelease,) $(if $(filter 1,$(RELEASE_DRAFT)),--draft,) $(if $(filter 1,$(RELEASE_PRUNE_EXTRA_ASSETS)),--prune-extra-assets,)

build-release-slim:
	@echo "══ 构建精简 Release 产物 ══"
	@$(RUSKTASK) build-release-slim

package-windows-installer:
	@echo "══ 打包 Windows 安装器 ══"
	@$(RUSKTASK) package-windows-installer $(if $(PACKAGE_VERSION),--version "$(PACKAGE_VERSION)",) $(if $(PACKAGE_BUILD_TAG),--build-tag "$(PACKAGE_BUILD_TAG)",) $(if $(filter 1,$(PACKAGE_SKIP_BUILD)),--skip-build,)

package-windows-assets:
	@echo "══ 打包 Windows 便携资产 ══"
	@$(RUSKTASK) package-windows-assets $(if $(PACKAGE_VERSION),--version "$(PACKAGE_VERSION)",) $(if $(PACKAGE_OUTPUT_DIR),--output-dir "$(PACKAGE_OUTPUT_DIR)",) $(if $(filter 1,$(PACKAGE_SKIP_BUILD)),--skip-build,)

package-windows-portable-installer:
	@echo "══ 打包 Windows 便携安装包（ZIP）══"
	@$(RUSKTASK) package-windows-portable-installer $(if $(PACKAGE_VERSION),--version "$(PACKAGE_VERSION)",) $(if $(PACKAGE_OUTPUT_DIR),--output-dir "$(PACKAGE_OUTPUT_DIR)",) $(if $(filter 1,$(PACKAGE_SKIP_BUILD)),--skip-build,)

release-local-windows:
	@echo "══ 本地 Windows Release 流程 ══"
	@$(SELF_MAKE) ci-local
	@$(SELF_MAKE) package-windows-installer PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_BUILD_TAG="$(PACKAGE_BUILD_TAG)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"
	@$(SELF_MAKE) package-windows-assets PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_OUTPUT_DIR="$(PACKAGE_OUTPUT_DIR)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"

release-local-linux:
	@echo "══ 本地 Linux Release 流程 ══"
	@$(SELF_MAKE) ci-local
	@echo "Linux .deb 本地打包脚本已在 v0.2.x 工作区移除，请改用 GitHub Release workflow 生成 .deb 产物" && exit 2

release-local-dual:
	@echo "══ 本地双系统 Release 流程 ══"
	@$(SELF_MAKE) release-local-windows PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_BUILD_TAG="$(PACKAGE_BUILD_TAG)" PACKAGE_OUTPUT_DIR="$(PACKAGE_OUTPUT_DIR)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"
	@$(SELF_MAKE) release-local-linux PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"

go-check:
	@echo "══ Go 编排检查链路 ══"
	@$(RUSKTASK) check

go-preflight:
	@echo "══ Go 编排发布前校验 ══"
	@$(RUSKTASK) preflight

go-build:
	@echo "══ Go 编排构建 ══"
	@$(RUSKTASK) build

go-test:
	@echo "══ Go 编排测试 ══"
	@$(RUSKTASK) test

go-doc:
	@echo "══ Go 编排文档构建 ══"
	@$(RUSKTASK) doc

go-audit:
	@echo "══ Go 编排安全审计 ══"
	@$(RUSKTASK) audit

go-release-sync:
	@echo "══ Go 编排发布状态审计 ══"
	@$(RUSKTASK) release-sync --mode audit

go-workflow-seal:
	@echo "══ Go 编排工作区固化审计 ══"
	@$(RUSKTASK) workflow-seal --mode audit

go-git-check:
	@echo "══ Go 编排 Git 工作流校验 ══"
	@$(RUSKTASK) git-check

go-rust-review:
	@echo "══ Go 编排 Rust 审查 ══"
	@$(RUSKTASK) rust-review

go-review:
	@echo "══ Go 编排综合审查 ══"
	@$(RUSKTASK) review

go-install-hooks:
	@echo "══ Go 编排安装 Git Hooks ══"
	@$(RUSKTASK) install-hooks

help:
	@echo "可用目标:"
	@echo "  ./scripts/ubuntu/task.sh check      格式 + clippy + 测试（快速校验）"
	@echo "  ./scripts/ubuntu/task.sh fmt        自动格式化全部代码"
	@echo "  ./scripts/ubuntu/task.sh fmt-check  检查格式（不修改）"
	@echo "  ./scripts/ubuntu/task.sh clippy     Clippy 静态分析"
	@echo "  ./scripts/ubuntu/task.sh test       运行全部测试"
	@echo "  ./scripts/ubuntu/task.sh test-release Release 模式测试"
	@echo "  ./scripts/ubuntu/task.sh build      Debug 构建正式发布项目"
	@echo "  ./scripts/ubuntu/task.sh release    Release 构建正式发布项目"
	@echo "  ./scripts/ubuntu/task.sh doc        生成文档"
	@echo "  ./scripts/ubuntu/task.sh audit      安全审计"
	@echo "  ./scripts/ubuntu/task.sh preflight  发布前完整校验"
	@echo "  ./scripts/ubuntu/task.sh ci-local  本地 CI 核心链路（governance + fmt/clippy/test/doc）"
	@echo "  ./scripts/ubuntu/task.sh ci-local-full  本地 CI 完整链路（ci-local + audit）"
	@echo "  ./scripts/ubuntu/task.sh release-sync  审计本地 release 状态一致性"
	@echo "  ./scripts/ubuntu/task.sh release-sync-apply  自动归一化本地 release 状态"
	@echo "  ./scripts/ubuntu/task.sh release-index  重建 release 索引表"
	@echo "  ./scripts/ubuntu/task.sh workflow-seal  一键执行清理+结构守卫+发布状态审计"
	@echo "  ./scripts/ubuntu/task.sh workflow-seal-apply  自动归一化并固化工作区状态"
	@echo "  ./scripts/ubuntu/task.sh workspace-guard  校验目录结构与路径策略"
	@echo "  ./scripts/ubuntu/task.sh workspace-cleanup  清理过程产物目录"
	@echo "  ./scripts/ubuntu/task.sh release-notes-validate RELEASE_NOTES_FILE=... RELEASE_NOTES_MODE=release|draft"
	@echo "  ./scripts/ubuntu/task.sh smart-bump BUMP_PART=patch BUMP_PUSH=1"
	@echo "  ./scripts/ubuntu/task.sh smart-rollback ROLLBACK_TAG=vX.Y.Z ROLLBACK_DELETE_REMOTE_TAG=1 ROLLBACK_DELETE_LOCAL_TAG=1"
	@echo "  ./scripts/ubuntu/task.sh pr-helper PR_CHECK=1"
	@echo "  ./scripts/ubuntu/task.sh git-check  Git 工作流校验"
	@echo "  ./scripts/ubuntu/task.sh git-review-before-push  推送前完整审查"
	@echo "  ./scripts/ubuntu/task.sh git-hooks-install|git-hooks-uninstall"
	@echo "  ./scripts/ubuntu/task.sh git-pr-check PR_BASE=main PR_HEAD=<branch>"
	@echo "  ./scripts/ubuntu/task.sh git-pr-create PR_BASE=main PR_HEAD=<branch> PR_TITLE='...'"
	@echo "  ./scripts/ubuntu/task.sh git-pr-merge PR_BASE=main PR_HEAD=<branch>"
	@echo "  ./scripts/ubuntu/task.sh docs-bundle DOCS_OUTPUT_ROOT=release_artifacts/docs-bundle DOCS_CREATE_ZIP=1"
	@echo "  ./scripts/ubuntu/task.sh release-publish RELEASE_TAG=vX.Y.Z RELEASE_OWNER=... RELEASE_REPO=..."
	@echo "  ./scripts/ubuntu/task.sh build-release-slim"
	@echo "  ./scripts/ubuntu/task.sh package-windows-installer PACKAGE_VERSION=X.Y.Z PACKAGE_BUILD_TAG=yyyymmdd PACKAGE_SKIP_BUILD=1"
	@echo "  ./scripts/ubuntu/task.sh package-windows-assets PACKAGE_VERSION=X.Y.Z PACKAGE_OUTPUT_DIR=release_artifacts PACKAGE_SKIP_BUILD=1"
	@echo "  ./scripts/ubuntu/task.sh package-windows-portable-installer PACKAGE_VERSION=X.Y.Z PACKAGE_OUTPUT_DIR=release_artifacts/windows-x64/installer PACKAGE_SKIP_BUILD=1"
	@echo "  ./scripts/ubuntu/task.sh release-local-windows PACKAGE_VERSION=X.Y.Z"
	@echo "  ./scripts/ubuntu/task.sh release-local-linux PACKAGE_VERSION=X.Y.Z"
	@echo "  ./scripts/ubuntu/task.sh release-local-dual PACKAGE_VERSION=X.Y.Z"
	@echo "  ./scripts/ubuntu/task.sh go-check  通过 Go 编排执行 check"
	@echo "  ./scripts/ubuntu/task.sh go-preflight  通过 Go 编排执行 preflight"
	@echo "  ./scripts/ubuntu/task.sh go-build  通过 Go 编排执行 build"
	@echo "  ./scripts/ubuntu/task.sh go-test  通过 Go 编排执行 test"
	@echo "  ./scripts/ubuntu/task.sh go-doc  通过 Go 编排执行 doc"
	@echo "  ./scripts/ubuntu/task.sh go-audit  通过 Go 编排执行 audit"
	@echo "  ./scripts/ubuntu/task.sh go-release-sync  通过 Go 编排执行 release-sync audit"
	@echo "  ./scripts/ubuntu/task.sh go-workflow-seal  通过 Go 编排执行 workflow-seal audit"
	@echo "  ./scripts/ubuntu/task.sh go-git-check  通过 Go 编排执行 git-check"
	@echo "  ./scripts/ubuntu/task.sh go-rust-review  通过 Go 编排执行 rust-review"
	@echo "  ./scripts/ubuntu/task.sh go-review  通过 Go 编排执行 review"
	@echo "  ./scripts/ubuntu/task.sh go-install-hooks  通过 Go 编排安装 Git hooks"
	@echo "  ./scripts/ubuntu/task.sh clean      清理所有 target/"
	@echo "  ./scripts/ubuntu/task.sh help       显示此帮助"


