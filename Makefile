# ============================================================
# robot_ctrl_rust_app Workspace — 统一构建自动化
# ============================================================
# 用法: make <target>
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

# Crate names (just package names, not paths)
ROBOT_CORE = robot_core
ROBOT_CONTROL = robot_control
TOOLS_SUITE = tools_suite
DEVTOOLS = devtools

ALL_CRATES = $(ROBOT_CORE) $(ROBOT_CONTROL) $(TOOLS_SUITE) $(DEVTOOLS)
BUILD_CRATES = $(ROBOT_CONTROL) $(TOOLS_SUITE)

# Parallel jobs (uses all available cores)
CARGO_JOBS := $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

.PHONY: all check fmt fmt-check clippy test test-release build release clean doc audit preflight help

all: check build

fmt:
	@echo "══ 格式化代码 ══"
	@cargo fmt

fmt-check:
	@echo "══ 检查格式 ══"
	@cargo fmt --check

clippy:
	@echo "══ Clippy 静态分析 ══"
	@cargo clippy --all-targets -- -D warnings

test:
	@echo "══ 运行测试 ══"
	@cargo test --all

test-release:
	@echo "══ Release 模式测试 ══"
	@cargo test --release --all

build:
	@echo "══ Debug 构建 (using $(CARGO_JOBS) jobs) ══"
	@cargo build -p $(ROBOT_CONTROL) -p $(TOOLS_SUITE)

build-parallel:
	@echo "══ Debug 并行构建 (using $(CARGO_JOBS) jobs) ══"
	@cargo build -p $(ROBOT_CONTROL) -p $(TOOLS_SUITE) -j $(CARGO_JOBS)

release:
	@echo "══ Release 构建 (using $(CARGO_JOBS) jobs) ══"
	@cargo build --release -p $(ROBOT_CONTROL) -p $(TOOLS_SUITE) -j $(CARGO_JOBS)
	@echo ""
	@echo "══ 构建产物 ══"
	@ls -lh target/release/$(ROBOT_CONTROL) target/release/$(TOOLS_SUITE) 2>/dev/null || true

release-parallel:
	@echo "══ Release 并行构建 (using $(CARGO_JOBS) jobs) ══"
	@cargo build --release -p $(ROBOT_CONTROL) -p $(TOOLS_SUITE) -j $(CARGO_JOBS)
	@echo ""
	@echo "══ 构建产物 ══"
	@ls -lh target/release/$(ROBOT_CONTROL) target/release/$(TOOLS_SUITE) 2>/dev/null || true

doc:
	@echo "══ 生成文档 ══"
	@RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

audit:
	@echo "══ 安全审计 ══"
	@for crate in $(ALL_CRATES); do \
		echo "→ $$crate"; \
		cargo audit -d $(AUDIT_DB) -f $$crate/Cargo.lock || exit 1; \
	done

clean:
	@echo "══ 清理构建产物 ══"
	@cargo clean

check: fmt-check clippy test
	@echo ""
	@echo "✓ 全部检查通过"

preflight: fmt-check clippy test test-release release doc
	@echo ""
	@echo "🚀 Preflight 全部通过，可以发布！"

# Parallel execution targets
check-parallel: fmt clippy test
	@echo ""
	@echo "✓ 全部检查通过 (parallel)"

build-all-parallel:
	@echo "══ 并行构建所有 crates (using $(CARGO_JOBS) jobs) ══"
	@cargo build --all -j $(CARGO_JOBS)

test-all-parallel:
	@echo "══ 并行测试所有 crates (using $(CARGO_JOBS) jobs) ══"
	@cargo test --all -j $(CARGO_JOBS)

help:
	@echo "可用目标:"
	@echo "  make check           格式 + clippy + 测试（快速校验）"
	@echo "  make check-parallel  格式 + clippy + 测试（并行）"
	@echo "  make fmt             自动格式化全部代码"
	@echo "  make fmt-check       检查格式（不修改）"
	@echo "  make clippy          Clippy 静态分析"
	@echo "  make test            运行全部测试"
	@echo "  make test-release    Release 模式测试"
	@echo "  make build           Debug 构建"
	@echo "  make build-parallel  Debug 并行构建"
	@echo "  make release         Release 构建"
	@echo "  make release-parallel Release 并行构建"
	@echo "  make build-all-parallel 并行构建所有 crates"
	@echo "  make test-all-parallel  并行测试所有 crates"
	@echo "  make doc             生成文档"
	@echo "  make audit           安全审计"
	@echo "  make preflight       发布前完整校验"
	@echo "  make clean           清理所有 target/"
	@echo "  make help            显示此帮助"