use habit_core::Severity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TriageResult {
    pub is_benign: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjusted_severity: Option<Severity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coaching_archetype: Option<String>,
}
