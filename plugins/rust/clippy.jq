def smell:
  . as $code
  | {
      "clippy::too_many_arguments": "too-many-parameters",
      "clippy::cognitive_complexity": "high-complexity",
      "clippy::cyclomatic_complexity": "high-complexity",
      "clippy::too_many_lines": "oversized-function",
      "unused_variables": "unused-variable",
      "unused_mut": "non-const-binding",
      "unused_imports": "unused-import",
      "dead_code": "unused-export",
      "clippy::match_wild_err_arm": "swallowed-exception",
      "clippy::let_underscore_must_use": "swallowed-exception",
      "clippy::undocumented_unsafe_blocks": "unsafe-usage",
      "clippy::wildcard_imports": "duplicate-import"
    }[$code] // $code;

group_by(.code | smell)
| map({
    smell: (.[0].code | smell),
    details: {},
    issues: map({
      key: .filename,
      details: {
        file: .filename,
        line: .line,
        column: .column,
        content: .content,
        message: .message,
        source: .code
      }
    })
  })
