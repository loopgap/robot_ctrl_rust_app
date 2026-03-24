# robot_control_rust v0.1.1

This maintenance update focuses on workflow reliability and release readiness.

## Highlights

- Unified local and CI validation coverage across `robot_control_rust`, `rust_micro_tools`, and all `rust_indie_tools/*` projects.
- Added actionable failure guidance for formatting, clippy, tests, docs, audit, deny, and release packaging steps.
- Fixed a rustdoc issue in the LQR model docs so strict docs builds now pass.
- Clarified release packaging so binaries are built and archived per project and per target.

## Validation

- Local `check` completed successfully across all tracked Rust projects.
- Local `doc` completed successfully after the rustdoc fix.
- Audit tooling is now part of the release-readiness workflow.
