# Distribution

This document describes the public distribution format and release-asset
verification for Unfour `v0.1.0 pre-release`.

## Release workflow

GitHub Actions runs the release workflow in three gates:

1. `verify` installs the frozen dependency graph and runs lint, unit tests,
   repository checks, Rust tests, and Playwright Chromium smoke tests.
2. The platform matrix builds the existing macOS and Linux targets. The
   Windows matrix keeps the shared Tauri `bundle.targets: "all"` configuration
   and stages both Windows installer formats.
3. `checksum-release` downloads all platform artifacts, generates one
   `SHA256SUMS.txt` from the actual files, and creates the pre-release with the
   installers and checksum manifest.

If `verify` fails, the build jobs do not run and no release assets are created.

For local builds, use `pnpm run tauri build`. On Windows, the shared
`bundle.targets: "all"` configuration produces both installer formats.

## Target artifacts

| Platform | Official distribution status | Format |
| --- | --- | --- |
| Windows x64 | Official pre-release distribution; choose one format | NSIS `.exe` or MSI `.msi` |
| macOS arm64/x64 | Experimental / unverified until real-device smoke checks | Existing Tauri `.dmg` and archive outputs |
| Linux x64 | Experimental / unverified until real-device smoke checks | Existing Tauri `.AppImage`, `.deb`, and available package outputs |

Windows NSIS and MSI install the same Unfour version. NSIS `.exe` is the
recommended choice for ordinary users. MSI `.msi` is provided for users who
prefer MSI or need software deployment management. Users should choose one.

Installing both formats on the same device is not recommended because it may
lead to duplicate desktop shortcuts, duplicate uninstall entries, and
confusing upgrade paths. Cross-format detection, automatic uninstall, and
NSIS/MSI cross-upgrade are not implemented at this stage.

The repository was checked for application code or installer hooks that create
desktop shortcuts. No such project-owned shortcut creation code was found.
The available user evidence shows two icons when NSIS and MSI are installed
together; this is attributed to the two installer packages, not to an extra
shortcut created by the application. Standalone NSIS and standalone MSI
shortcut counts still require separate clean-device smoke checks.

## Checksums

The final `checksum-release` job generates a single `SHA256SUMS.txt` using
`sha256sum` over the exact staged release assets. It uploads that file to the
same GitHub Release as the installers. Each line contains the SHA-256 followed
by the exact installer filename.

PowerShell can verify a downloaded Windows installer with:

```powershell
Get-FileHash -Algorithm SHA256 .\Unfour-*.exe
```

## Release caveats

- `v0.1.0` is an early pre-release and is not recommended for production use.
- Installers are unsigned and may trigger SmartScreen or other operating-system
  warnings.
- macOS and Linux must remain labeled experimental/unverified until real-device
  launch and smoke checks are recorded; a successful CI bundle build is not
  platform verification.
- Real SSH, PostgreSQL, MySQL/MariaDB, and system Keychain/Secret Service checks
  are not represented as automated passes unless they were run against those
  real systems.

## Installer smoke

For each platform that is claimed as verified, use a clean or disposable test
profile to install, launch, render the first viewport, exercise the documented
module navigation, quit and relaunch, and uninstall. Record OS warnings,
signing status, and upgrade behavior in the release verification matrix.
