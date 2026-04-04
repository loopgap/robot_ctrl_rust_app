# ============================================================
# rust_serial Workspace — 统一构建自动化
# ============================================================
# 用法:  make <target>
#   make check      — 格式 + clippy + 测试（快速校验）
#   make build      — Debug 构建正式发布项目
#   make release    — Release 构建正式发布项目
#   make clean      — 清理所有 target/
#   make fmt        — 自动格式化
#   make test       — 运行全部测试
#   make audit      — 安全审计
#   make doc        — 生成文档
#   make preflight  — 发布前完整校验
# ============================================================

SHELL := /bin/bash
AUDIT_DB := $(CURDIR)/.cargo-advisory-db

CORE_PROJECTS = robot_control_rust rust_tools_suite`nALL_PROJECTS := $(CORE_PROJECTS)

.PHONY: all check fmt fmt-check clippy test test-release build release clean doc audit preflight release-sync release-sync-apply workflow-seal workflow-seal-apply workspace-guard workspace-cleanup help

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
			echo "修改方向: 先格式化该项目，再重新运行 make fmt-check"; \
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

release-sync:
	@echo "══ Release 状态审计 ══"
	@pwsh -NoProfile -File scripts/cleanup-process-files.ps1 -Mode apply
	@pwsh -NoProfile -File scripts/enforce-workspace-structure.ps1 -Mode audit -Strict
	@pwsh -NoProfile -File scripts/sync-release-state.ps1 -Mode audit

release-sync-apply:
	@echo "══ Release 状态归一化 ══"
	@pwsh -NoProfile -File scripts/cleanup-process-files.ps1 -Mode apply
	@pwsh -NoProfile -File scripts/sync-release-state.ps1 -Mode apply -PruneLocalTagsNotOnRemote -CleanOrphanNotes
	@pwsh -NoProfile -File scripts/cleanup-process-files.ps1 -Mode apply
	@pwsh -NoProfile -File scripts/enforce-workspace-structure.ps1 -Mode audit -Strict

workflow-seal:
	@echo "══ Workflow 固化审计 ══"
	@pwsh -NoProfile -File scripts/workflow-seal.ps1 -Mode audit

workflow-seal-apply:
	@echo "══ Workflow 固化归一化 ══"
	@pwsh -NoProfile -File scripts/workflow-seal.ps1 -Mode apply -PruneLocalTagsNotOnRemote -CleanOrphanNotes

workspace-guard:
	@echo "══ Workspace 结构守卫 ══"
	@pwsh -NoProfile -File scripts/enforce-workspace-structure.ps1 -Mode audit -Strict

workspace-cleanup:
	@echo "══ Workspace 过程文件清理 ══"
	@pwsh -NoProfile -File scripts/cleanup-process-files.ps1 -Mode apply

help:
	@echo "可用目标:"
	@echo "  make check      格式 + clippy + 测试（快速校验）"
	@echo "  make fmt        自动格式化全部代码"
	@echo "  make fmt-check  检查格式（不修改）"
	@echo "  make clippy     Clippy 静态分析"
	@echo "  make test       运行全部测试"
	@echo "  make test-release Release 模式测试"
	@echo "  make build      Debug 构建正式发布项目"
	@echo "  make release    Release 构建正式发布项目"
	@echo "  make doc        生成文档"
	@echo "  make audit      安全审计"
	@echo "  make preflight  发布前完整校验"
	@echo "  make release-sync  审计本地 release 状态一致性"
	@echo "  make release-sync-apply  自动归一化本地 release 状态"
	@echo "  make workflow-seal  一键执行清理+结构守卫+发布状态审计"
	@echo "  make workflow-seal-apply  自动归一化并固化工作区状态"
	@echo "  make workspace-guard  校验目录结构与路径策略"
	@echo "  make workspace-cleanup  清理过程产物目录"
	@echo "  make clean      清理所有 target/"
	@echo "  make help       显示此帮助"


