pub mod control;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestEntry {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub version: u32,
    pub files: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VerificationFailure {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VerificationReport {
    pub checked: usize,
    pub valid: usize,
    pub failures: Vec<VerificationFailure>,
}

impl VerificationReport {
    pub fn is_ok(&self) -> bool {
        self.failures.is_empty()
    }
}

fn safe_relative_path(raw: &str) -> Result<PathBuf, String> {
    let path = Path::new(raw);
    if raw.is_empty() || path.is_absolute() {
        return Err("path must be a non-empty relative path".to_string());
    }
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("path must not contain traversal or platform prefix components".to_string());
    }
    Ok(path.to_path_buf())
}

fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn build_manifest(root: &Path, paths: &[String]) -> Result<Manifest, String> {
    if paths.is_empty() || paths.len() > 10_000 {
        return Err("between 1 and 10000 file paths are required".to_string());
    }

    let mut seen = HashSet::new();
    let mut files = Vec::with_capacity(paths.len());
    for raw in paths {
        let relative = safe_relative_path(raw)?;
        if !seen.insert(relative.clone()) {
            return Err(format!("duplicate path: {raw}"));
        }
        let full = root.join(&relative);
        let metadata = full
            .metadata()
            .map_err(|error| format!("{}: {error}", relative.display()))?;
        if !metadata.is_file() {
            return Err(format!("{}: not a regular file", relative.display()));
        }
        let digest =
            sha256_file(&full).map_err(|error| format!("{}: {error}", relative.display()))?;
        files.push(ManifestEntry {
            path: raw.clone(),
            bytes: metadata.len(),
            sha256: digest,
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Manifest { version: 1, files })
}

pub fn verify_manifest(root: &Path, manifest: &Manifest) -> VerificationReport {
    let mut failures = Vec::new();
    let mut valid = 0;
    let mut seen = HashSet::new();

    if manifest.version != 1 {
        failures.push(VerificationFailure {
            path: "<manifest>".to_string(),
            reason: format!("unsupported manifest version {}", manifest.version),
        });
    }

    for entry in &manifest.files {
        let relative = match safe_relative_path(&entry.path) {
            Ok(path) => path,
            Err(reason) => {
                failures.push(VerificationFailure {
                    path: entry.path.clone(),
                    reason,
                });
                continue;
            }
        };
        if !seen.insert(relative.clone()) {
            failures.push(VerificationFailure {
                path: entry.path.clone(),
                reason: "duplicate manifest path".to_string(),
            });
            continue;
        }

        let full = root.join(&relative);
        let metadata = match full.metadata() {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => {
                failures.push(VerificationFailure {
                    path: entry.path.clone(),
                    reason: "not a regular file".to_string(),
                });
                continue;
            }
            Err(error) => {
                failures.push(VerificationFailure {
                    path: entry.path.clone(),
                    reason: error.to_string(),
                });
                continue;
            }
        };

        if metadata.len() != entry.bytes {
            failures.push(VerificationFailure {
                path: entry.path.clone(),
                reason: format!(
                    "size mismatch: expected {}, got {}",
                    entry.bytes,
                    metadata.len()
                ),
            });
            continue;
        }

        match sha256_file(&full) {
            Ok(digest) if digest.eq_ignore_ascii_case(&entry.sha256) => valid += 1,
            Ok(digest) => failures.push(VerificationFailure {
                path: entry.path.clone(),
                reason: format!("sha256 mismatch: expected {}, got {digest}", entry.sha256),
            }),
            Err(error) => failures.push(VerificationFailure {
                path: entry.path.clone(),
                reason: error.to_string(),
            }),
        }
    }

    VerificationReport {
        checked: manifest.files.len(),
        valid,
        failures,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("sky-recovery-{nonce}"));
        fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn builds_and_verifies_manifest() {
        let root = test_root();
        fs::write(root.join("database.dump"), b"verified backup").expect("write fixture");
        let manifest = build_manifest(&root, &["database.dump".to_string()]).expect("manifest");
        let report = verify_manifest(&root, &manifest);
        assert!(report.is_ok());
        assert_eq!(report.valid, 1);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_changed_backup() {
        let root = test_root();
        fs::write(root.join("database.dump"), b"original").expect("write fixture");
        let manifest = build_manifest(&root, &["database.dump".to_string()]).expect("manifest");
        fs::write(root.join("database.dump"), b"changed!").expect("mutate fixture");
        let report = verify_manifest(&root, &manifest);
        assert!(!report.is_ok());
        assert_eq!(report.valid, 0);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn rejects_path_traversal() {
        let root = test_root();
        let error = build_manifest(&root, &["../secret".to_string()]).expect_err("must reject");
        assert!(error.contains("traversal"));
        fs::remove_dir_all(root).expect("cleanup");
    }
}
