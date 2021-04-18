{ pkgs ? import ../nix/pkgs.nix
, runTests ? true
}:
let
  sos21-backend = import ../. { inherit pkgs runTests; };
in
sos21-backend.sos21-database
