{ pkgs ? import ./nix/pkgs.nix }:
let
  sqlx-cli = pkgs.callPackage ./nix/sqlx-cli.nix { };
  crate2nix = import ./nix/crate2nix.nix { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    zlib
    rustc
    cargo
    sqlx-cli
    crate2nix
  ];
}
