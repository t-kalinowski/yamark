# CLI case files

Each `.case` file is executed through the compiled `yamark` binary. The format is:

```text
-- args
format --stdin-file-path input.md --wrap none
-- stdin
input text
-- stdout
expected stdout
-- stderr
expected stderr
-- status
0
```

The test harness intentionally avoids calling library internals. This keeps the
public CLI contract readable and prevents implementation details from becoming
part of the test API.

`markdown_shortcode_nested_skip_file_known_bug.case` records an existing bug:
a nested Markdown fence or div with `fmt: skip file` still passes through the
parent's whitespace cleanup, which removes trailing whitespace from shortcode
tokens and ordinary prose. Its expected output captures the current bug, not
the intended preservation contract. Fixing skipped-fragment emission is separate
from shortcode recognition.
