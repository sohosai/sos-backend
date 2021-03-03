{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "0pv3kh57mnia0w84hpbgwpp1p6d54x7nh0y1p46zxqhy67jalnd8";
}
