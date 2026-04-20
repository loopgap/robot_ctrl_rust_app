# Release Index

此文件由 scripts/go/rusktask update-release-index 生成，用于记录版本、Tag、本地/远端 Tag 状态与归档状态。

发布 `v0.2.1` 时，请额外核验：Tag 祖先归属 `origin/main`、`RELEASE_NOTES_v0.2.1.md` 通过结构校验、并确认 `.exe/.deb/checksums-sha256.txt` 资产齐全。

| Version | Tag | Local Tag Status | Remote Tag Status | Release Notes | Local Archive Status | Local Archive Path |
|---|---|---|---|---|---|---|
| 0.2.0 | v0.2.0 | present | unknown | - | not-archived | - |
| 0.1.8 | v0.1.8 | present | unknown | release_notes/RELEASE_NOTES_v0.1.8.md | not-archived | - |
| 0.1.7 | v0.1.7 | present | unknown | release_notes/RELEASE_NOTES_v0.1.7.md | archived | release_notes/archive_assets/v0.1.7 |
| 0.1.1 | v0.1.1 | present | unknown | release_notes/RELEASE_NOTES_v0.1.1.md | not-archived | - |
| 0.1.0 | v0.1.0 | present | unknown | release_notes/RELEASE_NOTES_v0.1.0.md | archived | release_notes/archive_assets/v0.1.0 |

更新时间(UTC): 2026-04-19 09:36:56