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

Recovered transcripts from PRs #4 and #6 are indexed in
[the reviewed-case inventory](../reviewed-cases/README.md), with historical
expectations and explanations of differences from the recorded main revision.
