# Release Checklist

This checklist is for the published `v0.1.0` release.

## Release candidate setup

- Confirm the release candidate commit and a clean working tree.
- Confirm the version remains `0.1.0` in the root package, desktop package,
  Tauri configuration, and any packaged Rust crates.
- Do not create, move, or rebuild the existing `v0.1.0` tag as part of release
  follow-up work.
- Review `README.md`, `README.zh-CN.md`, `CHANGELOG.md`, `SECURITY.md`, and
  `LICENSE`.
- Confirm release notes describe this as a release and do not claim
  unverified platforms or live-service checks are supported.

## Required automated verification

The release workflow must complete its independent `verify` job before any
platform build job:

- `pnpm install --frozen-lockfile`
- `pnpm run lint`
- `pnpm run test`
- `pnpm run check`
- `pnpm run test:rust`
- Playwright Chromium installation and `pnpm run test:e2e` when the GitHub
  Actions runner can execute the existing local smoke suite.

The workflow then builds macOS and Linux with their existing targets and keeps
the shared Windows `bundle.targets: "all"` configuration. The Windows release
asset set must contain both the NSIS `.exe` and MSI `.msi` for the same version.

## Artifact review

- Build artifacts come from the verified release candidate.
- The single aggregation job generates and uploads `SHA256SUMS.txt` alongside
  the installers.
- Artifact names identify the app, version, platform, and architecture where
  Tauri provides those fields.
- Windows release notes recommend NSIS for ordinary users and describe MSI as
  the option for MSI preference or software deployment management.
- Release notes tell users to choose one Windows format and warn against
  installing both on the same device.
- Unsigned artifacts and possible SmartScreen/security warnings are stated in
  the Release body.
- macOS/Linux artifacts remain experimental or unverified until real-device
  smoke checks are complete.

## Manual gates

- Windows NSIS install and app launch: record the actual result.
- Windows MSI install and app launch: record the actual result.
- On clean devices, verify that standalone NSIS and standalone MSI each create
  one desktop shortcut; verify that duplicate icons occur only when both
  installers are installed together.
- Windows first viewport, quit/relaunch, uninstall, and upgrade behavior:
  record the actual result; do not infer it from bundle generation.
- macOS and Linux launch/install smoke: `NOT VERIFIED` until run on real
  devices.
- API Client, Workspace, and MCP smoke: record only what was actually tested.
- Live SSH, PostgreSQL, MySQL/MariaDB, and system credential-store checks:
  require the corresponding real server, OS, or credential environment.
- Signing/notarization status: record as unsigned/not verified until completed.
- Do not expect cross-format detection, automatic uninstall, or NSIS/MSI
  cross-upgrade in this release.

## Go / no-go

Do not publish if a required automated verification step is `FAIL`. A
`NOT RUN` or `NOT VERIFIED` item requires maintainer acceptance; it must not be
rewritten as `PASS`.
