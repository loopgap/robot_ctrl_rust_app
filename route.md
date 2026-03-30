# 项目开发路线与规范 (Route.md)

> **最高原则**：统一覆盖、严格阻断、智能修复、极致性能。任何代码合并必须通过全量自动化验证。

---

## 1. 目录架构与分类规范

本工作区采用多项目协作模式，新增功能必须按类别归集：

- **`robot_control_rust/`**：核心控制引擎与高性能工业级 GUI。
  - 逻辑：`src/models/` (无 UI 逻辑), `src/services/` (通讯层)。
  - 视图：`src/views/` (基于 egui 的页面)。
- **`rust_micro_tools/`**：轻量级 TUI/CLI 工具，用于快速调试与系统诊断。
- **`rust_indie_tools/`**：单一功能的独立 GUI 工具（如 CSV 清理、JWT 解析）。
- **`docs/`**：基于 `mdBook` 的全局手册，记录用户指南与排障手册。
- **`scripts/`**：全工作区共享的自动化脚本系统。

---

## 2. 拓展与添加功能流程 (Standardized Workflow)

每个新功能或重大修改必须遵循以下生命周期：

### 2.1 调研与策略 (Research & Strategy)
- 检查 `robot_control_rust/ARCHITECTURE_AND_USAGE.md`，确保新功能符合现有架构模式。
- **性能约束**：优先考虑异步 (`tokio`) 与零拷贝解析 (`nom`)。UI 必须使用硬件加速的 `egui`。

### 2.2 实现与测试 (Implementation & Testing)
- **代码规范**：必须通过 `cargo fmt` 和 `cargo clippy`（严格模式）。
- **测试覆盖**：每个 `model` 或 `service` 必须包含单元测试。
- **国际化**：所有 UI 字符串必须集成到 `i18n.rs`，支持中英双语。

### 2.3 文档关联更新 (Documentation)
- **同步更新**：修改代码后，必须更新对应的子项目 `README.md`。
- **架构记录**：若涉及核心架构变更，必须更新 `robot_control_rust/ARCHITECTURE_AND_USAGE.md`。
- **手册同步**：如果是用户可见功能，需在 `docs/src/` 中增加相应章节。

---

## 3. 验证与自动化守则 (Validation Gate)

在提交任何更改前，开发者必须在本地运行以下验证：

```powershell
# 1. 快速检查
.\make.ps1 check

# 2. 推送前全量审查 (包含安全审计与跨项目测试)
.\scripts\review.ps1 -BeforePush
```

**严禁**跳过 `Git Hooks`。如果验证失败，必须根据“失败建议格式”进行智能修复后再提交。

---

## 4. 框架与标准指定

- **UI 框架**：统一使用 `egui` / `eframe` (即时模式 GUI)，禁止引入 Webview 或重型容器。
- **通讯标准**：基于串口、TCP、UDP、CAN 的通讯必须抽象为 `DataChannel` 模型。
- **提交规范**：遵循 `Conventional Commits` (feat, fix, docs, refactor, perf, test)。

---

## 5. 持续改进

本项目鼓励“文档即代码”。如果发现 `route.md` 或自动化脚本有不完善之处，应优先发起针对规范本身的优化 PR。

---

## 6. 发布治理规范 (Tag & Release Governance)

- **Tag 规范**：仅允许在 `main/master` 产生 `vMAJOR.MINOR.PATCH`（可扩展 `-rc.N` 预发布后缀）。
- **本地前置**：打 tag 前必须通过 `.\make.ps1 preflight`。
- **发布入口**：使用 `.\scripts\smart-bump.ps1` 完成升号、annotated tag 和发布说明草稿。
- **Release 必需资产**：
  - `robot_control_rust.exe`
  - `RobotControlSuite_Setup.exe`
  - `checksums-sha256.txt`
- **质量门禁**：Release 流水线必须包含 tag 策略校验、Windows 可执行文件 smoke test、哈希清单生成。
