use crate::guide::GuideResolver;
use habit_core::{Finding, ProjectConfig, Severity, SmellCatalogue, UncoachedHandling};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct MapperResult {
    pub exit_code: i32,
    pub output: String,
}

pub fn evaluate_and_render(
    input_str: &str,
    project_dir: &Path,
    plugin_dirs: &[PathBuf],
    config: &ProjectConfig,
) -> MapperResult {
    let trimmed = input_str.trim();

    // Stdin empty -> exit 2
    if trimmed.is_empty() {
        return MapperResult {
            exit_code: 2,
            output: "habit-mapper: nothing arrived on stdin — the sensors stage exited before it wrote any findings\n".to_string(),
        };
    }

    // Parse findings
    let findings: Vec<Finding> = match serde_json::from_str(trimmed) {
        Ok(f) => f,
        Err(err) => {
            return MapperResult {
                exit_code: 2,
                output: format!("habit-mapper: invalid JSON on stdin: {err}\n"),
            };
        }
    };

    // If findings empty array -> exit 0 with pass message
    if findings.is_empty() {
        return MapperResult {
            exit_code: 0,
            output: "✅ Habit Hooks: automated checks passed.\n".to_string(),
        };
    }

    // Filter and group findings by smell
    let mut custom_guides = BTreeMap::new();
    for (smell, cfg) in &config.smells {
        if let Some(ref g) = cfg.guide {
            custom_guides.insert(smell.clone(), g.clone());
        }
    }

    let guide_resolver = GuideResolver::new(project_dir, plugin_dirs, &custom_guides);

    let mut active_findings: Vec<(Finding, Severity)> = Vec::new();

    for finding in findings {
        if finding.issues.is_empty() {
            continue;
        }

        // Check if smell is disabled
        if let Some(smell_cfg) = config.smells.get(&finding.smell) {
            if smell_cfg.disabled.unwrap_or(false) {
                continue;
            }
        }

        // Incomplete run is always enforced
        if finding.smell == "incomplete-run" {
            active_findings.push((finding, Severity::Enforced));
            continue;
        }

        // Determine severity
        let severity = if let Some(smell_cfg) = config.smells.get(&finding.smell) {
            if let Some(sev) = smell_cfg.severity {
                Some(sev)
            } else if let Some(sev) = finding
                .details
                .get("severity")
                .and_then(|v| serde_json::from_value::<Severity>(v.clone()).ok())
            {
                Some(sev)
            } else {
                resolve_smell_severity(&finding.smell, config.uncoached)
            }
        } else if let Some(sev) = finding
            .details
            .get("severity")
            .and_then(|v| serde_json::from_value::<Severity>(v.clone()).ok())
        {
            Some(sev)
        } else {
            resolve_smell_severity(&finding.smell, config.uncoached)
        };

        if let Some(sev) = severity {
            active_findings.push((finding, sev));
        }
    }

    if active_findings.is_empty() {
        return MapperResult {
            exit_code: 0,
            output: "✅ Habit Hooks: automated checks passed.\n".to_string(),
        };
    }

    // Render guides
    let mut rendered_outputs = Vec::new();
    let mut has_enforced = false;

    for (finding, severity) in &active_findings {
        if *severity == Severity::Enforced {
            has_enforced = true;
        }

        match guide_resolver.render(finding) {
            Ok(rendered) => {
                if !rendered.trim().is_empty() {
                    rendered_outputs.push(rendered.trim().to_string());
                }
            }
            Err(err) => {
                rendered_outputs.push(format!("### {}\n(error rendering guide: {})", finding.smell, err));
            }
        }
    }

    let full_output = format!("{}\n", rendered_outputs.join("\n\n"));
    let exit_code = if has_enforced { 1 } else { 0 };

    MapperResult {
        exit_code,
        output: full_output,
    }
}

fn resolve_smell_severity(smell: &str, uncoached: UncoachedHandling) -> Option<Severity> {
    match smell {
        SmellCatalogue::TOO_MANY_PARAMETERS
        | SmellCatalogue::HIGH_COMPLEXITY
        | SmellCatalogue::OVERSIZED_FUNCTION
        | SmellCatalogue::OVERSIZED_FILE
        | SmellCatalogue::CYCLIC_DEPENDENCY
        | SmellCatalogue::CHANGE_RISK_HOTSPOT
        | SmellCatalogue::SWALLOWED_EXCEPTION
        | SmellCatalogue::DUPLICATE_DEPENDENCY
        | SmellCatalogue::INCOMPLETE_RUN
        | SmellCatalogue::PARSE_ERROR
        | SmellCatalogue::LOCK_ACROSS_AWAIT
        | SmellCatalogue::UNDOCUMENTED_UNSAFE => Some(Severity::Enforced),

        SmellCatalogue::UNUSED_VARIABLE
        | SmellCatalogue::NON_CONST_BINDING
        | SmellCatalogue::UNUSED_IMPORT
        | SmellCatalogue::UNUSED_FILE
        | SmellCatalogue::UNUSED_DEPENDENCY
        | SmellCatalogue::UNUSED_EXPORT
        | SmellCatalogue::TEST_ONLY_DEAD_CODE
        | SmellCatalogue::WARNING_COMMENT
        | SmellCatalogue::NON_ESSENTIAL_COMMENT
        | SmellCatalogue::DUPLICATED_CODE
        | SmellCatalogue::UNHANDLED_UNWRAP
        | SmellCatalogue::REDUNDANT_CLONE
        | SmellCatalogue::SUBOPTIMAL_BORROW_TYPE => Some(Severity::Suggested),

        _ => match uncoached {
            UncoachedHandling::Ignore => None,
            UncoachedHandling::Enforce => Some(Severity::Enforced),
            UncoachedHandling::Suggest => Some(Severity::Suggested),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod severity_evaluation {
        use super::*;
        use habit_core::{Issue, IssueDetails};
        use pretty_assertions::assert_eq;

        #[test]
        fn returns_exit_2_on_empty_input() {
            let res = evaluate_and_render("", Path::new("."), &[], &ProjectConfig::default());
            assert_eq!(res.exit_code, 2);
            assert_eq!(res.output.contains("nothing arrived on stdin"), true);
        }

        #[test]
        fn returns_exit_0_on_empty_findings_array() {
            let res = evaluate_and_render("[]", Path::new("."), &[], &ProjectConfig::default());
            assert_eq!(res.exit_code, 0);
            assert_eq!(res.output.contains("✅ Habit Hooks: automated checks passed."), true);
        }

        #[test]
        fn returns_exit_1_on_enforced_finding() {
            let json = serde_json::to_string(&vec![Finding {
                smell: "too-many-parameters".to_string(),
                language: Some("rust".to_string()),
                details: serde_json::json!({}),
                issues: vec![Issue {
                    key: "src/lib.rs".to_string(),
                    details: IssueDetails {
                        file: "src/lib.rs".to_string(),
                        line: Some(10),
                        column: None,
                        content: None,
                        message: Some("too many args".to_string()),
                        source: None,
                        extra: BTreeMap::new(),
                    },
                }],
            }])
            .unwrap();

            let res = evaluate_and_render(&json, Path::new("."), &[], &ProjectConfig::default());
            assert_eq!(res.exit_code, 1);
            assert_eq!(res.output.contains("too-many-parameters"), true);
        }

        #[test]
        fn returns_exit_0_on_suggested_finding() {
            let json = serde_json::to_string(&vec![Finding {
                smell: "unused-variable".to_string(),
                language: Some("rust".to_string()),
                details: serde_json::json!({}),
                issues: vec![Issue {
                    key: "src/lib.rs".to_string(),
                    details: IssueDetails {
                        file: "src/lib.rs".to_string(),
                        line: Some(10),
                        column: None,
                        content: None,
                        message: Some("variable x unused".to_string()),
                        source: None,
                        extra: BTreeMap::new(),
                    },
                }],
            }])
            .unwrap();

            let res = evaluate_and_render(&json, Path::new("."), &[], &ProjectConfig::default());
            assert_eq!(res.exit_code, 0);
            assert_eq!(res.output.contains("unused-variable"), true);
        }
    }
}
