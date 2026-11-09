<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

## Command line

### YAML formatting

- A file-scoped `fmt: template.delimiters` directive placed after the YAML it applies to now preserves template-containing values, as it does when placed before them. Previously, a late directive could split a flow collection such as `[<< a,b >>]` at the template's internal comma. This also applies to YAML inside Markdown fences.

### Embedded formatters

- An embedded formatter that exits successfully may now close stdin before reading all of its input. Yamark previously reported a broken-pipe error for large inputs. The formatter's output still receives normal validation, and other stdin write errors still fail.
