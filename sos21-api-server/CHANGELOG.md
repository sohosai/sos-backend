# Changelog

All notable changes to the `sos21-api-server` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and `sos21-api-server` adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
with the HTTP API interface being the "public API".

## [Unreleased]

### Added

- Support for non-ASCII filenames ([#98](https://github.com/sohosai/sos21-backend/pull/98))
- Editing of forms and form answers ([#90](https://github.com/sohosai/sos21-backend/pull/90))

### Changed

- Enable to specify project by code in file-distribution/create ([#96](https://github.com/sohosai/sos21-backend/pull/96))
- Do not require affiliation of graduate students and academic staffs ([#93](https://github.com/sohosai/sos21-backend/pull/93))

### Deprecated
### Removed
### Fixed
### Security

## [0.4.0-beta] - 2021-05-12

### Added

- Subowner ([#54](https://github.com/sohosai/sos21-backend/pull/54))
- Users' category ([#57](https://github.com/sohosai/sos21-backend/pull/57))
- Registration form ([#73](https://github.com/sohosai/sos21-backend/pull/73))
- User invitation ([#79](https://github.com/sohosai/sos21-backend/pull/79))

### Changed

- Change the specification around text and integer placeholders ([#65](https://github.com/sohosai/sos21-backend/pull/65))
- Rename some endpoints ([#76](https://github.com/sohosai/sos21-backend/pull/76))
- Restrict users to be assigned to only one project ([#78](https://github.com/sohosai/sos21-backend/pull/78))
    - `me/project/list` and `me/pending-project/list` are renamed to `me/project/get` and `me/pending-project/get` respectively
    - `pending-project/accept-subowner` is renamed to `project/create`

### Fixed

- 500 when an empty query is used in the form condition ([#63](https://github.com/sohosai/sos21-backend/pull/63), [#75](https://github.com/sohosai/sos21-backend/pull/75))

## [0.3.0] - 2021-03-29

### Added

- File sharings ([#42](https://github.com/sohosai/sos21-backend/pull/42))

### Removed

- Options to specify how the checkbox answers are displayed when exporting form answers ([#48](https://github.com/sohosai/sos21-backend/pull/48))

### Fixed

- Include CORS headers on errors as well ([#44](https://github.com/sohosai/sos21-backend/pull/44))
- Preserve filenames in the file download ([#46](https://github.com/sohosai/sos21-backend/pull/46))

### Security

- Update dependencies ([#47](https://github.com/sohosai/sos21-backend/pull/47))

## [0.2.1] - 2021-03-21

No changes in `sos21-api-server`.

## [0.2.0] - 2021-03-21

### Added

- Project code ([#20](https://github.com/sohosai/sos21-backend/pull/20), [#25](https://github.com/sohosai/sos21-backend/pull/25))
- File uploads ([#31](https://github.com/sohosai/sos21-backend/pull/31))
- Enable to participate in the CORS ([#37](https://github.com/sohosai/sos21-backend/pull/37))

### Changed

- Use millisecond timestamp instead of RFC3339 in the response ([#33](https://github.com/sohosai/sos21-backend/pull/33))
- Enrich meta endpoints ([#38](https://github.com/sohosai/sos21-backend/pull/38/files))
- Rename `/me` to `/me/get` ([#39](https://github.com/sohosai/sos21-backend/pull/39))

### Removed

- Display ID of projects ([#20](https://github.com/sohosai/sos21-backend/pull/20), [#25](https://github.com/sohosai/sos21-backend/pull/25))
