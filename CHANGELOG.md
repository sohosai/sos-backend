# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added

- File sharings ([#42](https://github.com/sohosai/sos21-backend/pull/42))

### Changed

- Adjust file usage quota ([#43](https://github.com/sohosai/sos21-backend/pull/43))
- Accept all email address with 'tsukuba.ac.jp' suffix as university email address ([#45](https://github.com/sohosai/sos21-backend/pull/45))

### Deprecated
### Removed

- Options to specify how the checkbox answers are displayed when exporting form answers ([#48](https://github.com/sohosai/sos21-backend/pull/48))

### Fixed

- Include CORS headers on errors as well ([#44](https://github.com/sohosai/sos21-backend/pull/44))
- Preserve filenames in the file download ([#46](https://github.com/sohosai/sos21-backend/pull/46))

### Security

## [0.2.1] - 2021-03-21

### Fixed

- Bundle the CA certificates in the Docker image ([e704d2d](https://github.com/sohosai/sos21-backend/commit/e704d2dd4ebb11cf6a4c0ebf3c7199d63bfb4a9d))

## [0.2.0] - 2021-03-21

### Added

- Project code ([#20](https://github.com/sohosai/sos21-backend/pull/20), [#25](https://github.com/sohosai/sos21-backend/pull/25))
- File uploads ([#31](https://github.com/sohosai/sos21-backend/pull/31))
- Enable to participate in the CORS ([#37](https://github.com/sohosai/sos21-backend/pull/37))

### Changed

- Use millisecond timestamp instead of RFC3339 in the response ([#33](https://github.com/sohosai/sos21-backend/pull/33))
- Enrich meta endpoints ([#38](https://github.com/sohosai/sos21-backend/pull/38/files))

### Removed

- Display ID of projects ([#20](https://github.com/sohosai/sos21-backend/pull/20), [#25](https://github.com/sohosai/sos21-backend/pull/25))

### Fixed

- Inflexibility of `sos21-database` ([#28](https://github.com/sohosai/sos21-backend/pull/28))
- Server error logs ([#36](https://github.com/sohosai/sos21-backend/pull/36))
