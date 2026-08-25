# Sky Recovery

Sky Recovery is a small Rust CLI/library for creating and verifying backup manifests. It records the relative path, byte size, and SHA-256 digest of explicitly selected backup artifacts, then verifies those artifacts before a restore or recovery exercise.

**Status: engineering beta.** This repository verifies backup-file integrity; it does not perform backups, restore databases, orchestrate failover, manage cloud snapshots, or prove a disaster-recovery plan will meet an RPO/RTO.

## Create a manifest

```bash
sky-recovery create /srv/backups database.dump config.tar > recovery-manifest.json
```

Only explicit relative file paths are accepted. Absolute paths, `..` traversal, duplicate paths, directories, empty path lists, and manifests above 10,000 entries are rejected.

## Verify a manifest

```bash
sky-recovery verify recovery-manifest.json /srv/backups
```

Verification checks manifest version, path safety, duplicate entries, regular-file type, exact byte size, and SHA-256 digest. The command exits `0` only when every listed artifact verifies; integrity failures exit `1`; invalid invocation/configuration exits `2`.

## Local verification

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo audit
cargo build --release
```

Container:

```bash
docker build -t sky-recovery .
docker run --rm sky-recovery --help
```

The image runs as numeric non-root UID `10001`. CI verifies formatting, compilation, Clippy, unit tests, dependency audit, release build, image build, non-root configuration, and CLI startup.

## SKYCOIN4444 integration

Sky Recovery can be used by infrastructure/runbook automation as a pre-restore integrity check. A deployment should generate manifests alongside backup artifacts, store them in an independently protected location, and verify them before restore drills. Integration should invoke the CLI or library contract rather than copying implementation code.

## Security and recovery limits

A checksum proves byte integrity against the supplied manifest, not authenticity by itself. This beta does not sign manifests, encrypt backups, protect credentials, validate application-level consistency, test restore procedures, replicate data, manage retention, or measure RPO/RTO. Production disaster recovery requires independently protected/signed metadata, actual restore drills, infrastructure controls, observability, access control, and documented incident procedures.
