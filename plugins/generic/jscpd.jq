def base_name:
  split("/") | last;

[
  .duplicates[]?
  | . as $dup
  | (
      {
        file: $dup.firstFile.name,
        line: $dup.firstFile.startLoc.line,
        column: $dup.firstFile.startLoc.column,
        message: ("Duplicate code block (" + ($dup.lines | tostring) + " lines, " + ($dup.tokens | tostring) + " tokens) matches " + ($dup.secondFile.name | base_name) + ":" + ($dup.secondFile.startLoc.line | tostring))
      },
      {
        file: $dup.secondFile.name,
        line: $dup.secondFile.startLoc.line,
        column: $dup.secondFile.startLoc.column,
        message: ("Duplicate code block (" + ($dup.lines | tostring) + " lines, " + ($dup.tokens | tostring) + " tokens) matches " + ($dup.firstFile.name | base_name) + ":" + ($dup.firstFile.startLoc.line | tostring))
      }
    )
]
| if length == 0 then []
  else
    [
      {
        smell: "duplicated-code",
        details: {},
        issues: map({
          key: .file,
          details: {
            file: .file,
            line: .line,
            column: .column,
            message: .message,
            source: "jscpd"
          }
        })
      }
    ]
  end
