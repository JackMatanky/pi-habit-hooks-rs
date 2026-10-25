group_by(.kind)
| map({
    smell: .[0].kind,
    details: {},
    issues: map({
      key: .file,
      details: {
        file: .file,
        line: .line,
        column: (.column // null),
        content: .content,
        message: .message,
        source: "rust-comment"
      }
    })
  })
