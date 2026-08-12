# Releasing Yamark

Yamark keeps its development version at the most recently released version plus `+dev`. The current version must end in `+dev` before starting a release. Run releases from a clean, up-to-date `main` checkout with authenticated `git` and `gh` access.

The examples below release `0.4.0`. Pass versions to the helper without a `v` prefix.

## Prepare the release

1. Fetch `main` and tags, then confirm that the checkout is clean, `HEAD` matches `origin/main`, and `v0.4.0` does not exist locally or remotely.
2. Review the changes since the previous tag and replace `RELEASE_NOTES.md` with the notes for the new version. Its first sentence must start with `Yamark 0.4.0 `. Put command-line changes first and editor-only changes in a separate section, then format the file with `yamark format --wrap paragraph RELEASE_NOTES.md`.
3. Update all five version locations:

   ```sh
   scripts/set-version.py 0.4.0
   ```

4. Run the same checks as CI:

   ```sh
   scripts/check.sh
   ```

5. Inspect the complete diff. Commit the five version files and `RELEASE_NOTES.md` as `Prepare 0.4.0 release`, then push `main`.
6. Stop and wait for CI and Pages to pass for the exact release commit before creating the tag.

## Publish the release

After CI and Pages pass, confirm that `main` and `origin/main` still point to the release commit. Create and push the tag at that exact commit:

```sh
git tag v0.4.0 RELEASE_COMMIT
git push origin refs/tags/v0.4.0
```

The tag-driven release workflow validates all five versions and the release-note version. It builds four native archives, four wheels, and one source distribution; smoke-tests the wheels; creates the GitHub release with `RELEASE_NOTES.md`; and publishes the Python distributions to PyPI through Trusted Publishing.

Stop and wait for the Release workflow to finish. Then:

1. Confirm that the GitHub release is public and is neither a draft nor a prerelease.
2. Download all four native archives and `SHA256SUMS`, then verify every checksum.
3. Confirm that PyPI has four wheels and one source distribution, and that each file has provenance for `t-kalinowski/yamark`, `release.yml`, and the `pypi` environment.
4. Test the public package without using a local checkout or cache:

   ```sh
   uvx --isolated --no-cache --no-sources yamark@0.4.0 --help
   printf 'items: [one,two]\n' | uvx --isolated --no-cache --no-sources yamark@0.4.0 format --stdin-file-path input.yaml --verify
   printf '{"items":["one","two"]}\n' | uvx --isolated --no-cache --no-sources yamark@0.4.0 to-yaml --stdin-file-path input.json
   ```

## Return to development

Once the public release checks pass, mark subsequent builds as development builds:

```sh
scripts/set-version.py 0.4.0+dev
scripts/check.sh
git add Cargo.toml Cargo.lock pyproject.toml uv.lock editors/vscode/package.json
git commit -m "Mark post-release builds as development versions"
git push origin main
```

Confirm that `main` and `origin/main` match, while `v0.4.0` still points to the release commit. Stop and wait for CI and Pages on the development-version commit.

## Failure boundary

The process follows one path and stops at the first failure. It does not undo or resume partial releases automatically. Inspect the commits, tag, Actions runs, release assets, PyPI project, and version files before continuing manually.

Do not rerun the release from the beginning after its tag exists. If PyPI publication fails while the workflow artifacts still exist, rerun the failed job so it uses the original distributions. PyPI filenames are immutable; if an uploaded distribution cannot be completed from the original artifacts, release a new version. Complete the `+dev` commit manually after recovering a partial release.
