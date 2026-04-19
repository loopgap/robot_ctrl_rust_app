# ============================================================
# rust_serial Workspace — 统一构建自动化
# ============================================================
# 用法:  ./scripts/task <target>（或 make -f scripts/Makefile <target>）
#   ./scripts/task check      — 格式 + clippy + 测试（快速校验）
#   ./scripts/task build      — Debug 构建正式发布项目
#   ./scripts/task release    — Release 构建正式发布项目
#   ./scripts/task clean      — 清理所有 target/
#   ./scripts/task fmt        — 自动格式化
#   ./scripts/task test       — 运行全部测试
#   ./scripts/task audit      — 安全审计
#   ./scripts/task doc        — 生成文档
#   ./scripts/task preflight  — 发布前完整校验
# ============================================================

SHELL := /bin/bash
AUDIT_DB := $(CURDIR)/.cargo-advisory-db
RUSKTASK := cd scripts/go/rusktask && go run .

CORE_PROJECTS = robot_control_rust rust_tools_suite
ALL_PROJECTS := $(CORE_PROJECTS)
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
		cargo fmt --manifest-path $${p}Cargo.toml || exit 1; \
	done

fmt-check:
	@echo "══ 检查格式 ══"
	@for p in $(ALL_PROJECTS); do \
		echo "→ $$p"; \
		if ! cargo fmt --check --manifest-path $${p}Cargo.toml; then \
			echo ""; \
			echo "问题摘要: $$p 存在未格式化代码"; \
			echo "建议命令: cargo fmt --manifest-path $${p}Cargo.toml"; \
			echo "修改方向: 先格式化该项目，再重新运行 ./scripts/task fmt-check"; \
			echo "如需继续排查先看哪里: $$p 下最近修改的 .rs 文件"; \
			exit 1; \
		fi; \
	done

clippy:
	@echo "══ Clippy 静态分析 ══"
	@for p in $(ALL_PROJECTS); do \
		echo "→ $$p"; \
		if ! cargo clippy --manifest-path $${p}Cargo.toml --all-targets -- -D warnings; then \
			echo ""; \
			echo "问题摘要: $$p 的 Clippy 检查失败"; \
			echo "建议命令: cargo clippy --manifest-path $${p}Cargo.toml --all-targets -- -D warnings"; \
			echo "修改方向: 优先修复 -D warnings 触发项，再处理具体 lint"; \
			echo "如需继续排查先看哪里: Clippy 日志中首个 error 或 warning 所在文件"; \
			exit 1; \
		fi; \
	done

test:
	@echo "══ 运行测试 ══"
	@for p in $(ALL_PROJECTS); do \
		echo "→ $$p"; \
		if ! cargo test --manifest-path $${p}Cargo.toml; then \
			echo ""; \
			echo "问题摘要: $$p 的测试失败"; \
			echo "建议命令: cargo test --manifest-path $${p}Cargo.toml"; \
			echo "修改方向: 先复现失败用例，再区分断言失败、夹具问题或平台差异"; \
			echo "如需继续排查先看哪里: 失败测试名称和对应模块"; \
			exit 1; \
		fi; \
	done

test-release:
	@echo "══ Release 模式测试 ══"
	@for p in $(ALL_PROJECTS); do \
		echo "→ $$p"; \
		if ! cargo test --release --manifest-path $${p}Cargo.toml; then \
			echo ""; \
			echo "问题摘要: $$p 的 Release 测试失败"; \
			echo "建议命令: cargo test --release --manifest-path $${p}Cargo.toml"; \
			echo "修改方向: 检查优化级别下的行为差异、特性门和初始化路径"; \
			echo "如需继续排查先看哪里: Release 模式下失败的测试用例"; \
			exit 1; \
		fi; \
	done

build:
	@echo "══ Debug 构建 ══"
	@for p in $(CORE_PROJECTS); do \
		echo "→ $$p"; \
		if ! cargo build --manifest-path $${p}Cargo.toml; then \
			echo ""; \
			echo "问题摘要: $$p 的 Debug 构建失败"; \
			echo "建议命令: cargo build --manifest-path $${p}Cargo.toml"; \
			echo "修改方向: 先修复编译错误，再确认依赖和特性配置"; \
			echo "如需继续排查先看哪里: 编译日志中首个 error"; \
			exit 1; \
		fi; \
	done

release:
	@echo "══ Release 构建 ══"
	@for p in $(CORE_PROJECTS); do \
		echo "→ $$p"; \
		if ! cargo build --release --manifest-path $${p}Cargo.toml; then \
			echo ""; \
			echo "问题摘要: $$p 的 Release 构建失败"; \
			echo "建议命令: cargo build --release --manifest-path $${p}Cargo.toml"; \
			echo "修改方向: 检查系统依赖、目标平台配置和 release-only 分支"; \
			echo "如需继续排查先看哪里: release 构建日志中首个 error"; \
			exit 1; \
		fi; \
	done
	@echo ""
	@echo "══ 构建产物 ══"
	@ls -lh robot_control_rust/target/release/robot_control_rust* 2>/dev/null || true
	@ls -lh rust_tools_suite/target/release/rust_tools_suite* 2>/dev/null || true

doc:
	@echo "══ 生成文档 ══"
	@for p in $(ALL_PROJECTS); do \
		echo "→ $$p"; \
		if ! RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --manifest-path $${p}Cargo.toml; then \
			echo ""; \
			echo "问题摘要: $$p 的文档构建失败"; \
			echo "建议命令: RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --manifest-path $${p}Cargo.toml"; \
			echo "修改方向: 先修复 rustdoc warning 和失效文档注释"; \
			echo "如需继续排查先看哪里: rustdoc 输出中的 warning 或 error"; \
			exit 1; \
		fi; \
	done

audit:
	@echo "══ 安全审计 ══"
	@for p in $(ALL_PROJECTS); do \
		echo "→ $$p"; \
		if ! cargo audit -d $(AUDIT_DB) -f $${p}Cargo.lock; then \
			echo ""; \
			echo "问题摘要: $$p 的安全审计失败"; \
			echo "建议命令: cargo audit -d $(AUDIT_DB) -f $${p}Cargo.lock"; \
			echo "修改方向: 优先升级存在漏洞的依赖，必要时重新生成 Cargo.lock"; \
			echo "如需继续排查先看哪里: audit 输出中的 advisory ID 和受影响依赖"; \
			exit 1; \
		fi; \
		if ! (cd $$p && cargo deny check advisories bans sources --config $(CURDIR)/deny.toml); then \
			echo ""; \
			echo "问题摘要: $$p 的依赖策略检查失败"; \
			echo "建议命令: (cd $$p && cargo deny check advisories bans sources --config $(CURDIR)/deny.toml)"; \
			echo "修改方向: 检查 advisories、sources 和 bans 规则是否需要调整"; \
			echo "如需继续排查先看哪里: cargo-deny 输出中的首个 error"; \
			exit 1; \
		fi; \
	done

clean:
	@echo "══ 清理构建产物 ══"
	@for p in $(ALL_PROJECTS); do \
		cargo clean --manifest-path $${p}Cargo.toml; \
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
	@$(MAKE) fmt-check
	@$(MAKE) clippy
	@$(MAKE) test
	@$(MAKE) doc

ci-local-full: ci-local
	@echo "══ 本地 CI（完整）══"
	@$(MAKE) audit

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
	@$(MAKE) ci-local
	@$(MAKE) package-windows-installer PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_BUILD_TAG="$(PACKAGE_BUILD_TAG)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"
	@$(MAKE) package-windows-assets PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_OUTPUT_DIR="$(PACKAGE_OUTPUT_DIR)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"

release-local-linux:
	@echo "══ 本地 Linux Release 流程 ══"
	@$(MAKE) ci-local
	@bash rust_tools_suite/packaging/package_deb.sh $(if $(PACKAGE_VERSION),--version "$(PACKAGE_VERSION)",) $(if $(filter 1,$(PACKAGE_SKIP_BUILD)),--skip-build,)

release-local-dual:
	@echo "══ 本地双系统 Release 流程 ══"
	@$(MAKE) release-local-windows PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_BUILD_TAG="$(PACKAGE_BUILD_TAG)" PACKAGE_OUTPUT_DIR="$(PACKAGE_OUTPUT_DIR)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"
	@$(MAKE) release-local-linux PACKAGE_VERSION="$(PACKAGE_VERSION)" PACKAGE_SKIP_BUILD="$(PACKAGE_SKIP_BUILD)"

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
	@echo "  ./scripts/task check      格式 + clippy + 测试（快速校验）"
	@echo "  ./scripts/task fmt        自动格式化全部代码"
	@echo "  ./scripts/task fmt-check  检查格式（不修改）"
	@echo "  ./scripts/task clippy     Clippy 静态分析"
	@echo "  ./scripts/task test       运行全部测试"
	@echo "  ./scripts/task test-release Release 模式测试"
	@echo "  ./scripts/task build      Debug 构建正式发布项目"
	@echo "  ./scripts/task release    Release 构建正式发布项目"
	@echo "  ./scripts/task doc        生成文档"
	@echo "  ./scripts/task audit      安全审计"
	@echo "  ./scripts/task preflight  发布前完整校验"
	@echo "  ./scripts/task ci-local  本地 CI 核心链路（governance + fmt/clippy/test/doc）"
	@echo "  ./scripts/task ci-local-full  本地 CI 完整链路（ci-local + audit）"
	@echo "  ./scripts/task release-sync  审计本地 release 状态一致性"
	@echo "  ./scripts/task release-sync-apply  自动归一化本地 release 状态"
	@echo "  ./scripts/task release-index  重建 release 索引表"
	@echo "  ./scripts/task workflow-seal  一键执行清理+结构守卫+发布状态审计"
	@echo "  ./scripts/task workflow-seal-apply  自动归一化并固化工作区状态"
	@echo "  ./scripts/task workspace-guard  校验目录结构与路径策略"
	@echo "  ./scripts/task workspace-cleanup  清理过程产物目录"
	@echo "  ./scripts/task release-notes-validate RELEASE_NOTES_FILE=... RELEASE_NOTES_MODE=release|draft"
	@echo "  ./scripts/task smart-bump BUMP_PART=patch BUMP_PUSH=1"
	@echo "  ./scripts/task smart-rollback ROLLBACK_TAG=vX.Y.Z ROLLBACK_DELETE_REMOTE_TAG=1 ROLLBACK_DELETE_LOCAL_TAG=1"
	@echo "  ./scripts/task pr-helper PR_CHECK=1"
	@echo "  ./scripts/task git-check  Git 工作流校验"
	@echo "  ./scripts/task git-review-before-push  推送前完整审查"
	@echo "  ./scripts/task git-hooks-install|git-hooks-uninstall"
	@echo "  ./scripts/task git-pr-check PR_BASE=main PR_HEAD=<branch>"
	@echo "  ./scripts/task git-pr-create PR_BASE=main PR_HEAD=<branch> PR_TITLE='...'"
	@echo "  ./scripts/task git-pr-merge PR_BASE=main PR_HEAD=<branch>"
	@echo "  ./scripts/task docs-bundle DOCS_OUTPUT_ROOT=robot_control_rust/dist/windows-x64/docs-bundle DOCS_CREATE_ZIP=1"
	@echo "  ./scripts/task release-publish RELEASE_TAG=vX.Y.Z RELEASE_OWNER=... RELEASE_REPO=..."
	@echo "  ./scripts/task build-release-slim"
	@echo "  ./scripts/task package-windows-installer PACKAGE_VERSION=X.Y.Z PACKAGE_BUILD_TAG=yyyymmdd PACKAGE_SKIP_BUILD=1"
	@echo "  ./scripts/task package-windows-assets PACKAGE_VERSION=X.Y.Z PACKAGE_OUTPUT_DIR=release_artifacts PACKAGE_SKIP_BUILD=1"
	@echo "  ./scripts/task package-windows-portable-installer PACKAGE_VERSION=X.Y.Z PACKAGE_OUTPUT_DIR=robot_control_rust/dist/windows-x64/installer PACKAGE_SKIP_BUILD=1"
	@echo "  ./scripts/task release-local-windows PACKAGE_VERSION=X.Y.Z"
	@echo "  ./scripts/task release-local-linux PACKAGE_VERSION=X.Y.Z"
	@echo "  ./scripts/task release-local-dual PACKAGE_VERSION=X.Y.Z"
	@echo "  ./scripts/task go-check  通过 Go 编排执行 check"
	@echo "  ./scripts/task go-preflight  通过 Go 编排执行 preflight"
	@echo "  ./scripts/task go-build  通过 Go 编排执行 build"
	@echo "  ./scripts/task go-test  通过 Go 编排执行 test"
	@echo "  ./scripts/task go-doc  通过 Go 编排执行 doc"
	@echo "  ./scripts/task go-audit  通过 Go 编排执行 audit"
	@echo "  ./scripts/task go-release-sync  通过 Go 编排执行 release-sync audit"
	@echo "  ./scripts/task go-workflow-seal  通过 Go 编排执行 workflow-seal audit"
	@echo "  ./scripts/task go-git-check  通过 Go 编排执行 git-check"
	@echo "  ./scripts/task go-rust-review  通过 Go 编排执行 rust-review"
	@echo "  ./scripts/task go-review  通过 Go 编排执行 review"
	@echo "  ./scripts/task go-install-hooks  通过 Go 编排安装 Git hooks"
	@echo "  ./scripts/task clean      清理所有 target/"
	@echo "  ./scripts/task help       显示此帮助"


