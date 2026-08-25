# Changelog

## 0.1.0 - 2026-08-24

- replace unrelated in-memory time-series behavior with backup manifest creation and verification
- add safe relative-path validation, duplicate detection, byte-size checks, and SHA-256 integrity verification
- add deterministic JSON reports and meaningful CLI exit codes
- add Rustfmt, Clippy, tests, dependency audit, release build, non-root image, and CLI smoke CI gates
- remove unrelated Node packaging metadata
- document that checksum verification is not a substitute for signed metadata, restore drills, or a complete disaster-recovery program
