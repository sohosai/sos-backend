{ pkgs ? import ./nix/pkgs.nix }:
let
  sqlx-cli = pkgs.callPackage ./nix/sqlx-cli.nix { };
  rustPlatform = pkgs.callPackage ./nix/rustPlatform.nix { };
in
pkgs.mkShell {
  nativeBuildInputs = [
    rustPlatform.rust.rustc
    rustPlatform.rust.cargo
    sqlx-cli
    pkgs.zlib
  ];
}
