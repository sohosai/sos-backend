{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "00x7px65r5180sq8fcwzqmksxwwd9w92mrmi5bwkn9b1sba441nf";
}
