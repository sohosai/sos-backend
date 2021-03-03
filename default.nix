{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "1d0y79i81jxg7ypsa08z6x896vghbfzkz3q21f6xijw1m48hmj8n";
}
