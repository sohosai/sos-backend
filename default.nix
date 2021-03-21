{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.1";
  src = ./.;

  cargoSha256 = "1m4l0bmbp4izp69kmaj4ibggasvin40qg9rf7n5chq2nyhm3kqa4";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
