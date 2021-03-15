{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "0idfhsh9kxj644g0j5b1gmv9hqa5099mnw90gc2lpzd6cv6wplkh";
}
