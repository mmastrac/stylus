# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **SNMP Monitor**: New built-in SNMP monitor that automates network device monitoring
  - Support for complex SNMP queries with filtering and expressions
  - Configurable include/exclude rules for interface selection
  - Expression-based status evaluation (green/red conditions)
  - Automatic interface discovery and monitoring
  - Support for custom SNMP community strings and targets
  - Built-in functions for string operations (startswith, contains, etc.)
  - Comprehensive documentation and examples

### Changed
- Enhanced running documentation

### Fixed
- Dependency updates and security patches

## [0.14.0] - 2025-08-01

### Changed
- Updated documentation with improved running instructions
- Cleaned up path handling for better cross-platform compatibility

### Fixed
- Fixed stylus-ui dependency issues
- Fixed stylus-ui build process
- Resolved dependency conflicts and build errors

### Dependencies
- Bumped all dependencies to latest compatible versions

## [0.13.0] - 2025-07-28

### Added
- **New React-based UI**: Complete rewrite of the user interface
  - Modern React-based frontend with TypeScript
  - Built-in visualizations: iframe, isoflow, stack view, and more
  - No longer requires end-users to provide HTML for most cases
  - Improved user experience with better navigation and styling

### Changed
- **Major Architecture Change**: Restructured project into workspace with separate crates
  - Split into `stylus` (backend) and `stylus-ui` (frontend) crates
  - Improved build process with automatic UI bundling
  - Better separation of concerns between backend and frontend
- Enhanced documentation with new screenshots and examples
- Improved example configurations and templates

### Dependencies
- Major dependency updates for improved performance and security
- Updated to newer versions of tokio, axum, and other core dependencies

## [0.12.1] - 2025-07-XX

### Fixed
- Better error messages in Docker environment
- Improved documentation and examples

## [0.12.0] - 2025-07-XX

### Added

 - Added `stylus init` command
 - Migrate `--test` and `--dump` to named subcommands (`stylus test`, etc)
 - Improved static file serving
 - Add a fallback `index.html` rendering if one doesn't exist

### Changed
- Better Docker support and error handling

---

## Migration Notes

### From v0.12.x to v0.13.0
- **Breaking Change**: The UI system has been completely rewritten
- HTML-based customizations may need to be updated for the new React-based system
- New configuration options available for enhanced visualizations
- Improved monitor grouping and display capabilities

### From v0.13.x to v0.14.0
- No breaking changes
- Improved path handling for better cross-platform support
- Enhanced documentation and examples

### From v0.14.x to Unreleased
- **New Feature**: SNMP monitor support for network device monitoring
- Enhanced expression system for complex monitoring logic
- Improved documentation for advanced use cases
