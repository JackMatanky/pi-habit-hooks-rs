def smell:
  . as $kind
  | {
      "unused": "unused-dependency",
      "duplicate": "duplicate-dependency"
    }[$kind] // $kind;

group_by(.kind | smell)
| map({
    smell: (.[0].kind | smell),
    details: {},
    issues: map({
      key: .name,
      details: {
        file: .file,
        line: (.line // null),
        message: .message,
        source: ("cargo-deps:" + .kind)
      }
    })
  })
