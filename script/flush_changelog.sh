#! /usr/bin/env nix-shell
#! nix-shell -i bash --pure
# shellcheck shell=bash

# flush the changelogs in the project.
#
# usage:
#   ./script/flush_changelog.sh <new_version>

set -feu -o pipefail

PROJECT=$(git rev-parse --show-toplevel)
readonly PROJECT=${PROJECT%/}
readonly SCRIPT=$0

TEMPLATE=$(cat <<'EOT'
## [Unreleased]

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security
EOT
)
readonly TEMPLATE

function main() {
  set -feu -o pipefail

  if [ $# != 1 ]; then
    1>&2 echo "usage: $SCRIPT <new_version>"
    return 1
  fi

  local -r version=$1
  local date
  date=$(date +'%Y-%m-%d')
  readonly date

  1>&2 echo "version: $version"
  1>&2 echo "date: $date"

  local -a files
  IFS=$'\n' read -r -d '' -a files < <(git ls-files | grep 'CHANGELOG.md$'; printf '\0')
  readonly files

  if [ ${#files[*]} -eq 0 ]; then
    1>&2 echo "no CHANGELOG.md found"
    return 1
  fi

  for file in "${files[@]}"; do
    sed -i -e '/^###/{:b;N;/### \w*$/{D;bb}}' -e '/^###/{:c;N;/\n$/bc;s/.*\n##/##/}' "$file"
    sed -i -e 's/^## \[Unreleased\]$/'"${TEMPLATE//$'\n'/'\n'}\n\n## [$version] - $date/" "$file"
  done

  { echo "updated:"; printf '%s\n' "${files[@]}"; } 1>&2
}

main "$@"
