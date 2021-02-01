{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "0v02263jh65np6p1agfdk2vq7g2q4y0x9hqkn86z1khii74bdhsj";
}
