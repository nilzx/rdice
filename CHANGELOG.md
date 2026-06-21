# Changelog

All notable changes to this workspace will be documented here.

This project follows SemVer for each published package.

## Unreleased

- No unreleased changes.

## 0.1.2 - 2026-06-21

- Added colored CLI output with `--no-color` and `NO_COLOR` support.
- Added TUI color styling with `NO_COLOR` support.
- Improved TUI custom dice creation with a step-by-step input wizard.
- Added a direct TUI shortcut for opening the custom dice manager.
- Fixed TUI overview alignment for wide text faces such as Chinese dice faces.

## 0.1.1 - 2026-06-21

- Organized the public workspace documentation.
- Added MIT License metadata.
- Added crate-level README files for `rdice-core`, `rdice-cli`, and `rdice-tui`.
- Improved TUI pagination across tray, dice, and add-die managers.
- Added persistent TUI roll history.
- Clarified command-mode and custom-face behavior, including `+` as text and
  `+1` as numeric input in TUI commands.
- Compacted large built-in numeric dice in CLI list output.
