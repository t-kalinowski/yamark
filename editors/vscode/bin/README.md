# Bundled Yamark Binaries

Put platform-specific Yamark executables under:

```text
bin/<platform>-<arch>/yamark
bin/<platform>-<arch>/yamark.exe
```

Examples:

```text
bin/darwin-arm64/yamark
bin/darwin-x64/yamark
bin/linux-arm64/yamark
bin/linux-x64/yamark
bin/win32-x64/yamark.exe
```

Set `yamark.useBundledExecutable` to `true` to use the bundled executable. The
extension default is `false` because public packages may not include binaries
for every platform.

For local development, `npm run build:dev` copies the current platform's
`target/release/yamark` here before packaging the VSIX.
