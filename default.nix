{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "1w2arkf991vgk2138py39bcr0pqpsy397f2090ycgsr7qqp7lniq";
}
