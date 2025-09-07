# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.16.4] - 2025-09-07

### Fixed
- **Web UI**: Fixed status indicators in the web UI during initial load

## [0.16.3] - 2025-09-07

### Changed
- **Status JSON**: Config and status JSON are split for better loading performance

## [0.16.2] - 2025-09-07

### Changed
- **Logs**: Logs are no longer transmitted in status.json to minimize updates
- **Table Visualization**: Table visualization groups status indicators by state (except red)

## [0.16.1] - 2025-09-07

### Changed
- **Fullscreen Mode**: Improved fullscreen mode to be more responsive and consistent

## [0.16.0] - 2025-09-05

### Added
- **Row Visualization**: New built-in row visualization that allows you to layout status indicators side-by-side

### Changed
- **Logo**: New project logo
- **Stack Visualization**
 - Layout groups can now be laid out column-wise
 - Ordering of status indicators can now be controlled
- **SNMP Monitor**:
 - Retry improvements

## [0.15.1] - 2025-09-04

### Changed
- **SNMP Monitor**: Relevant SNMP MIB data is shipped with Stylus so MIB files are not required

## [0.15.0] - 2025-08-10

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
