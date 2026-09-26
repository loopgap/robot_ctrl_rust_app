# v0.1.8

## Highlights

- 收录 v0.1.7 修复后的可运行 GUI 基线，完成工作区级统一构建验证。
- 修复 v0.1.7 版本资产中存在的启动即退出问题，确保 `robot_control_rust.exe` 在 Windows 上稳定运行。
- 统一 workspace CI 检查路径，使多项目结构下的 fmt/clippy/test 验证对齐。

## Fixes

- 修复 Release workflow 产物上传路径与资产命名不一致问题。
- 修复 workspace 结构守卫路径误报。

## Verification

- [x] `cargo build --release`
- [x] `cargo test`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] Local smoke equivalent passed
