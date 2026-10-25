map(select(.lines > (.threshold // 500)))
| if length == 0 then []
  else
    [
      {
        smell: "oversized-file",
        details: {},
        issues: map({
          key: .file,
          details: {
            file: .file,
            line: .lines,
            message: ("File length (" + (.lines | tostring) + " lines) exceeds threshold of " + ((.threshold // 500) | tostring) + " lines"),
            source: "line-count"
          }
        })
      }
    ]
  end
