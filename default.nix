{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.1";
  src = ./.;

  cargoSha256 = "0q6mrwlc5r5fkhspdzglvd01y7dgd31fdvmrddma315pyvys6sg7";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
