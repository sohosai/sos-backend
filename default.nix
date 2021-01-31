{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "0d9yr0ggr8gbndnlbmy8dhnvhacqkzibjwzyhg6m0bsmc6h2ndn2";
}
