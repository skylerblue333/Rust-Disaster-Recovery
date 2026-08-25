# Security

Sky Recovery is an engineering-beta backup integrity verifier. Report suspected vulnerabilities privately to the repository owner rather than publishing exploit details in an issue.

## Current controls

- rejects absolute paths and traversal components
- rejects duplicate manifest entries
- bounds manifest creation to 10,000 explicit files
- verifies regular-file type, byte size, and SHA-256 digest
- dependency audit and Clippy run in CI
- container runs as non-root UID `10001`

## Important limits

SHA-256 verification against an untrusted manifest does not establish authenticity. This repository does not sign manifests, encrypt backups, protect credentials, enforce retention, validate database consistency, perform restores, manage cloud access, or test infrastructure failover. Production recovery workflows should protect manifests independently, restrict backup access, sign metadata, perform real restore drills, and monitor recovery objectives.
