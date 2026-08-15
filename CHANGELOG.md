# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-15

### Added

- `nitrile init` command for initializing new Git-based LaTeX projects, either
  from the built-in default preset or from a template.
  - `-T/--template` accepts a local directory or a remote Git URL (including
    scp-like `git@host:path` shorthand); remote templates are cloned and stripped
    of their `.git`, local templates are copied (skipping `.git` and symlinks).
  - `--force` allows initializing into a non-empty directory.
- `compile` now fails fast with a clear message when `pdflatex` is not present on
  `PATH`.
- On a failed compilation, the output points to the location of the `.log` file.

### Changed

- **Breaking:** the default compile output path is now `build/<target>.pdf`
  (previously the current directory).

### Build

- Added `cargo-deny` for dependency/license/advisory checking, wired into CI and
  the `prek` hooks.
- Resolved `clippy::pedantic` warnings and enabled the lints in CI.

## [0.1.0] - 2026-08-03

### Added

- Initial release.
- `compile` command for building LaTeX projects from the command line, with
  `-f/--flag` support for CLI-injected arguments and conditional compilation via
  the companion `nitrile.sty` package.
- Post-compile summary reporting page count and elapsed time.

[0.2.0]: https://github.com/cooper-schoone/nitrile/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cooper-schoone/nitrile/releases/tag/v0.1.0
