# yaml-test-suite data

This directory is the local fixture root for generated data from:
https://github.com/yaml/yaml-test-suite

`data/` and `License` are generated locally and ignored by git.

Populate from the local `r-yaml12` checkout:

```sh
tools/bootstrap-yaml-test-suite-data.py --source ~/github/posit-dev/r-yaml12/tests/testthat/yaml-test-suite
```

Or clone upstream and run its `make data` target:

```sh
tools/bootstrap-yaml-test-suite-data.py
YAML_TEST_SUITE_REF=<commit-ish> tools/bootstrap-yaml-test-suite-data.py
```

The generated data is available for local formatter experiments and targeted
regression tests.
