# Repository guides

For installation, CLI options, configuration, and supported syntax, start with the [user documentation](../website/reference.qmd). The Quarto sources in `website/` own those contracts.

| Contributor task                               | Guide                                             |
| ---------------------------------------------- | ------------------------------------------------- |
| Set up a checkout and choose validation        | [Development](DEVELOPMENT.md)                     |
| Find the implementation owner and invariants   | [Architecture](ARCHITECTURE.md)                   |
| Add a public regression or run a focused suite | [Tests](../tests/README.md)                       |
| Understand retained inline source              | [Inline literal preservation](inline-literals.md) |
| Record agreed follow-up work                   | [Open work](TODO.md)                              |
| Prepare and publish a release                  | [Release](../RELEASE.md)                          |
| Run benchmarks                                 | [Benchmark tools](../tools/bench/README.md)       |

[Design sketches](../design-sketches/README.md) retain completed workpads and dated experiments. Benchmark JSON under `docs/benchmarks/` records individual runs. Neither is a current behavior specification.
