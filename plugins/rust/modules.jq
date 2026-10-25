def smell:
  . as $kind
  | {
      "orphan": "unused-file",
      "cycle": "cyclic-dependency"
    }[$kind] // $kind;

group_by(.kind | smell)
| map({
    smell: (.[0].kind | smell),
    details: {},
    issues: map({
      key: .file,
      details: {
        file: .file,
        line: (.line // null),
        column: (.column // null),
        message: .message,
        source: ("cargo-modules:" + .kind)
      }
    })
  })
