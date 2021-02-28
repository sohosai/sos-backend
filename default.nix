{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "1nlc90sz380hc0113ilk9jlb7d6w6hqw82qb7w0d32xz65hxrxrp";
}
