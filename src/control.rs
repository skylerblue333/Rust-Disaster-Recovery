use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupPlan {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub retention_copies: u16,
    pub required_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BackupPlanSnapshot {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub retention_copies: u16,
    pub required_artifacts: Vec<String>,
    pub backup_performed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupEvidence {
    pub plan_id: String,
    pub manifest_verified: bool,
    pub artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BackupReadiness {
    pub ready: bool,
    pub missing_artifacts: Vec<String>,
    pub manifest_verified: bool,
    pub backup_performed: bool,
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
}

pub fn validate_backup_plan(plan: &BackupPlan) -> Result<BackupPlanSnapshot, String> {
    if !valid_id(&plan.id) || !valid_id(&plan.source_id) || !valid_id(&plan.target_id) {
        return Err("plan, source, and target identifiers must be bounded tokens".to_string());
    }
    if plan.source_id == plan.target_id {
        return Err("source and target identifiers must differ".to_string());
    }
    if !(1..=365).contains(&plan.retention_copies) {
        return Err("retention_copies must be between 1 and 365".to_string());
    }
    if plan.required_artifacts.is_empty() || plan.required_artifacts.len() > 128 {
        return Err("required_artifacts must contain between 1 and 128 entries".to_string());
    }

    let mut artifacts = Vec::with_capacity(plan.required_artifacts.len());
    let mut seen = HashSet::new();
    for artifact in &plan.required_artifacts {
        if !valid_id(artifact) {
            return Err("artifact identifiers must be bounded tokens".to_string());
        }
        if !seen.insert(artifact.clone()) {
            return Err(format!("duplicate artifact identifier: {artifact}"));
        }
        artifacts.push(artifact.clone());
    }
    artifacts.sort();

    Ok(BackupPlanSnapshot {
        id: plan.id.clone(),
        source_id: plan.source_id.clone(),
        target_id: plan.target_id.clone(),
        retention_copies: plan.retention_copies,
        required_artifacts: artifacts,
        backup_performed: false,
    })
}

pub fn evaluate_backup_readiness(
    plan: &BackupPlan,
    evidence: &BackupEvidence,
) -> Result<BackupReadiness, String> {
    let normalized = validate_backup_plan(plan)?;
    if evidence.plan_id != normalized.id {
        return Err("backup evidence does not match plan id".to_string());
    }
    if evidence.artifact_ids.len() > 256 {
        return Err("backup evidence contains too many artifacts".to_string());
    }

    let supplied: HashSet<&str> = evidence.artifact_ids.iter().map(String::as_str).collect();
    let missing_artifacts = normalized
        .required_artifacts
        .iter()
        .filter(|artifact| !supplied.contains(artifact.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    Ok(BackupReadiness {
        ready: evidence.manifest_verified && missing_artifacts.is_empty(),
        missing_artifacts,
        manifest_verified: evidence.manifest_verified,
        backup_performed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> BackupPlan {
        BackupPlan {
            id: "plan:daily".to_string(),
            source_id: "db:primary".to_string(),
            target_id: "store:archive".to_string(),
            retention_copies: 30,
            required_artifacts: vec!["database.dump".to_string(), "manifest.json".to_string()],
        }
    }

    #[test]
    fn normalizes_plan_and_never_claims_execution() {
        let mut input = plan();
        input.required_artifacts.reverse();
        let normalized = validate_backup_plan(&input).expect("valid plan");
        assert_eq!(normalized.required_artifacts, vec!["database.dump", "manifest.json"]);
        assert!(!normalized.backup_performed);
    }

    #[test]
    fn readiness_requires_manifest_and_artifacts() {
        let ready = evaluate_backup_readiness(
            &plan(),
            &BackupEvidence {
                plan_id: "plan:daily".to_string(),
                manifest_verified: true,
                artifact_ids: vec!["manifest.json".to_string(), "database.dump".to_string()],
            },
        )
        .expect("readiness");
        assert!(ready.ready);
        assert!(!ready.backup_performed);

        let missing = evaluate_backup_readiness(
            &plan(),
            &BackupEvidence {
                plan_id: "plan:daily".to_string(),
                manifest_verified: true,
                artifact_ids: vec!["manifest.json".to_string()],
            },
        )
        .expect("readiness");
        assert!(!missing.ready);
        assert_eq!(missing.missing_artifacts, vec!["database.dump"]);
    }

    #[test]
    fn rejects_unsafe_plan_inputs() {
        let mut input = plan();
        input.source_id = input.target_id.clone();
        assert!(validate_backup_plan(&input).unwrap_err().contains("must differ"));

        let mut duplicate = plan();
        duplicate.required_artifacts = vec!["manifest.json".into(), "manifest.json".into()];
        assert!(validate_backup_plan(&duplicate).unwrap_err().contains("duplicate"));
    }
}
