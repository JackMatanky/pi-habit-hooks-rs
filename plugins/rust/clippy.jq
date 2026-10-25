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
      "clippy::undocumented_unsafe_blocks": "undocumented-unsafe",
      "clippy::wildcard_imports": "duplicate-import",
      "clippy::await_holding_lock": "lock-across-await",
      "clippy::await_holding_refcell_ref": "lock-across-await",
      "clippy::unwrap_used": "unhandled-unwrap",
      "clippy::expect_used": "unhandled-unwrap",
      "clippy::panic": "unhandled-unwrap",
      "clippy::redundant_clone": "redundant-clone",
      "clippy::clone_on_ref_ptr": "redundant-clone",
      "clippy::clone_on_copy": "redundant-clone",
      "clippy::ptr_arg": "suboptimal-borrow-type",
      "clippy::needless_borrow": "suboptimal-borrow-type",
      "clippy::needless_collect": "intermediate-allocation"
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
