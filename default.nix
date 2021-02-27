{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "19ll1lnfv9bs960cznh68j2wkg58kxm2qn09chmjr131dyngcnrk";
}
