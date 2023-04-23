# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security

## [0.7.0] - 2023-04-23

### Added
- Planning attributes for indoor projects.
- Slack notification function for any suspicious JWT access.

### Changed
- Switch to `sos-backend-infrasturucture` from `sos22` one.
- Updated to nixpkgs 22.11.

### Removed
- Online Planning category has been removed.

## [0.6.2] - 2022-06-26

### Fixed

- Avoid project index collision by replacing `count_projects` with `get_next_index`([#158](https://github.com/sohosai/sos21-backend/pull/161))

## [0.6.1] - 2022-06-22

### Added

- Send Slack notification when a new form answer is submitted
- Enable `workflow_dispatch` event on Beta/Main Workflows
- `/get-project-creation-availability` endpoint to check if it's in a project creation period ([#157](https://github.com/sohosai/sos21-backend/pull/157))
- Store `exceptional_complete_deadline` and apply it to prepare project or to answer registration form([#158](https://github.com/sohosai/sos21-backend/pull/158))

## [0.6.0] - 2022-04-13

### Changed

- Append online `ProjectCategory` entries ([#140](https://github.com/sohosai/sos21-backend/pull/140))
- Project Code composition
- Switch to `sos22-backend-infrasturucture` from `sos21` one.

### Removed

- `affiliation` field for user ([#139](https://github.com/sohosai/sos21-backend/pull/139))

### Fixed

- Switch to 2022 configuration set([#138](https://github.com/sohosai/sos21-backend/pull/138))

## [0.5.3] - 2021-09-29

### Fixed

- Enable to upload files larger than 10MB ([#125](https://github.com/sohosai/sos21-backend/pull/125))

## [0.5.2] - 2021-08-28

### Added

- `updated_at` field ([#118](https://github.com/sohosai/sos21-backend/pull/118))

### Changed

- Enable to set project creation period by categories ([#118](https://github.com/sohosai/sos21-backend/pull/118))

### Fixed

- Accept empty answer in !accept_multiple_files file form item ([#120](https://github.com/sohosai/sos21-backend/pull/120))

### Security

- Update `tokio` to 1.8.1 ([#114](https://github.com/sohosai/sos21-backend/pull/114))
- Update `hyper` to 0.14.12 ([#121](https://github.com/sohosai/sos21-backend/pull/121))
- Update `rusoto_*` to 0.47.0 ([#122](https://github.com/sohosai/sos21-backend/pull/122))

## [0.5.1] - 2021-06-16

### Added

- `project_query` file sharing ([#104](https://github.com/sohosai/sos21-backend/pull/104))
- `has_answer` flag for project list endpoints ([#106](https://github.com/sohosai/sos21-backend/pull/106))
- `/assign-user-role-to-email` endpoint to ensure the email is assigned the specific role ([#109](https://github.com/sohosai/sos21-backend/pull/109))

### Fixed

- Expose `Content-Disposition` header in the CORS ([#105](https://github.com/sohosai/sos21-backend/pull/105))

### Security

- Replace `apply-macro` with `macro_rules_attribute` since `apply-macro` is yanked (#[107](https://github.com/sohosai/sos21-backend/pull/107))

## [0.5.0-beta] - 2021-05-23

### Added

- Support for non-ASCII filenames ([#98](https://github.com/sohosai/sos21-backend/pull/98))
- Pre-configured administrator email address ([#99](https://github.com/sohosai/sos21-backend/pull/99))
- Editing of forms and form answers ([#90](https://github.com/sohosai/sos21-backend/pull/90))
- Editing of registration form answer and projects ([#100](https://github.com/sohosai/sos21-backend/pull/100))

### Changed

- Relax checkbox form item constraints ([#95](https://github.com/sohosai/sos21-backend/pull/95))
- Enable to specify project by code when distributing files ([#96](https://github.com/sohosai/sos21-backend/pull/96))
- Do not require affiliation of graduate students and academic staffs ([#93](https://github.com/sohosai/sos21-backend/pull/93))

## [0.4.0-beta] - 2021-05-12

### Added

- Subowner ([#54](https://github.com/sohosai/sos21-backend/pull/54))
- Users' category ([#57](https://github.com/sohosai/sos21-backend/pull/57))
- Registration form ([#73](https://github.com/sohosai/sos21-backend/pull/73))
- User invitation ([#79](https://github.com/sohosai/sos21-backend/pull/79))

### Changed

- Use GitHub Container Registry instead of GitHub Packages Docker registry to store container images ([#55](https://github.com/sohosai/sos21-backend/pull/55))
- Use crate2nix for building ([#59](https://github.com/sohosai/sos21-backend/pull/59))
- Change the specification around text and integer placeholders ([#65](https://github.com/sohosai/sos21-backend/pull/65))
- Change the specification around the project name and description length ([#69](https://github.com/sohosai/sos21-backend/pull/69))
- Require stripped texts ([#70](https://github.com/sohosai/sos21-backend/pull/70))
- Refactor endpoint and API specs ([#76](https://github.com/sohosai/sos21-backend/pull/76))
- Restrict users to be assigned to only one project ([#78](https://github.com/sohosai/sos21-backend/pull/78))

### Fixed

- Exit with non-zero exit code for errors ([#62](https://github.com/sohosai/sos21-backend/pull/62))
- 500 when an empty query is used in the form condition ([#63](https://github.com/sohosai/sos21-backend/pull/63), [#75](https://github.com/sohosai/sos21-backend/pull/75))
- Fix is_health query to check some missing tables ([#66](https://github.com/sohosai/sos21-backend/pull/66))

## [0.3.0] - 2021-03-29

### Added

- File sharings ([#42](https://github.com/sohosai/sos21-backend/pull/42))

### Changed

- Adjust file usage quota ([#43](https://github.com/sohosai/sos21-backend/pull/43))
- Accept all email address with 'tsukuba.ac.jp' suffix as university email address ([#45](https://github.com/sohosai/sos21-backend/pull/45))

### Removed

- Options to specify how the checkbox answers are displayed when exporting form answers ([#48](https://github.com/sohosai/sos21-backend/pull/48))

### Fixed

- Include CORS headers on errors as well ([#44](https://github.com/sohosai/sos21-backend/pull/44))
- Preserve filenames in the file download ([#46](https://github.com/sohosai/sos21-backend/pull/46))

### Security

- Update dependencies ([#47](https://github.com/sohosai/sos21-backend/pull/47))

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
