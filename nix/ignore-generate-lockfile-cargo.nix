{ pkgs ? import ./pkgs.nix }:
pkgs.writeShellScriptBin "cargo" ''
  if [ "$1" == "generate-lockfile" ]; then
    1>&2 echo 'Ignoring `cargo generate-lockfile`'
    exit 0
  fi
  ${pkgs.cargo}/bin/cargo "$@"
''
