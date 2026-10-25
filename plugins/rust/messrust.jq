def smell:
  . as $rule
  | {
      "ExcessiveParameterList": "too-many-parameters",
      "CyclomaticComplexity": "high-complexity",
      "NPathComplexity": "high-complexity",
      "ExcessiveMethodLength": "oversized-function",
      "ExcessiveClassLength": "oversized-file",
      "TooManyFields": "too-many-fields",
      "TooManyMethods": "too-many-methods",
      "TooManyPublicMethods": "over-exposed-api",
      "ExcessiveClassComplexity": "high-complexity"
    }[$rule] // $rule;

[
  .files[]? | . as $file
  | .violations[]?
  | {
      smell: (.rule | smell),
      file: $file.file,
      line: .beginLine,
      message: .description,
      rule: .rule
    }
]
| group_by(.smell)
| map({
    smell: .[0].smell,
    details: {},
    issues: map({
      key: .file,
      details: {
        file: .file,
        line: .line,
        message: .message,
        source: ("messrust:" + .rule)
      }
    })
  })
