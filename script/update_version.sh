#! /usr/bin/env nix-shell
#! nix-shell -i bash --pure
# shellcheck shell=bash

# update version numbers in the project.
#
# usage:
#   ./script/update_version.sh <new_version>

set -feu -o pipefail

PROJECT=$(git rev-parse --show-toplevel)
readonly PROJECT=${PROJECT%/}
readonly SCRIPT=$0

function update_cargo_version() {
  set -feu -o pipefail

  local -r version=$1

  pushd "$PROJECT" &> /dev/null
  # TODO: Update value of 'package.version' precisely
  find . -name 'Cargo.toml' -exec sed -i '0,/^version = "[^"]*"$/s//version = "'"$version"'"/' {} \;
  # TODO: find lightweight way to just update Cargo.lock from Cargo.toml
  cargo check -q
  crate2nix generate > /dev/null
  popd &> /dev/null
}

function update_schema_version() {
  set -feu -o pipefail

  local -r version=$1

  pushd "$PROJECT/sos21-api-server/schema/" &> /dev/null
  yq write -i api.yml 'info.version' "$version"
  tmp_json=$(mktemp)
  jq ".version = \"$version\"" < package.json > "$tmp_json"
  mv "$tmp_json" package.json
  npm install --package-lock-only --no-audit > /dev/null
  popd &> /dev/null
}

function update_docker_image_version() {
  set -feu -o pipefail

  local -r version=$1
  local -r compose_yml=$PROJECT/docker-compose.run.yml

  local image_name
  image_name=$(yq read "$compose_yml" 'services.api-server.image')
  yq write -i "$compose_yml" 'services.api-server.image' "${image_name/:v*/:v}$version"
  yq write -i "$compose_yml" 'services.run-migrations.image' "${image_name/:v*/:v}$version"
}

function main() {
  set -feu -o pipefail

  if ! git diff --quiet --exit-code; then
    1>&2 echo "working tree contains unstaged changes"
    return 1
  fi

  if [ $# != 1 ]; then
    1>&2 echo "usage: $SCRIPT <new_version>"
    return 1
  fi

  local -r version=$1
  1>&2 echo "version: $version"

  update_cargo_version "$version"
  update_schema_version "$version"
  update_docker_image_version "$version"

  { echo "updated:"; git --no-pager diff --name-only; } 1>&2
}

main "$@"
