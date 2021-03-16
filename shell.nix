{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
mkShell {
  nativeBuildInputs = [
    rustPlatform.rust.rustc
    rustPlatform.rust.cargo
    sqlx-cli
    zlib
  ];
}
