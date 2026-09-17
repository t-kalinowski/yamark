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

Recovered cases from PRs #4 and #6 are indexed in [the reviewed-case inventory](../reviewed-cases/README.md). Active cases record the specified main revision. Differing historical expectations live outside this harness and are not a second passing suite. Consult the inventory before changing a baseline that records an unresolved limitation.
