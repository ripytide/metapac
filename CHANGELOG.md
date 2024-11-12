# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## [0.2.2] - 2024-11-10

### Added

- Fixed build Errors and Commands not being found on Windows (#44)
- Added the WinGet Package Manager (#44)
- Added the HomeBrew Package Manager (#41)
- Added new test to de-duplicate the codebase by pulling the example config and
  group files directly from the README.md

### Fixed

- Fixed the optional dependencies Install Option in Arch packages being
  ignored (#39)
- Fixed Flatpak package runtimes not being detected (#40)

### Documentation

- Improved the config and group file location documentation (#44, #45)

## [0.2.1] - 2024-10-29

### Documentation

- Update cargo install command to the README.md
- Add AUR build install command README.md
- Rewrote CONTRIBUTING.md (#36)

### Added

- Added `pikaur` as another optional arch backend

### Fixed

- Fixed Install Options in group files being ignored (#30)

## [0.2.0] - 2024-10-20

### Build

- Adjust build automation

## [0.1.0] - 2024-10-20

### Added

- Initial Release
