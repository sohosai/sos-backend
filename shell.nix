{ pkgs ? import ./nix/pkgs.nix }:
let
  sos21-backend = import ./. { inherit pkgs; };
  sqlx-cli = pkgs.callPackage ./nix/sqlx-cli.nix { };
in
sos21-backend.overrideAttrs (oldAttrs: {
  nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
    sqlx-cli
  ];
})
