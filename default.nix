{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "14kpcw7vja12bfkmzpa90kvplkwhvw8m2dlnl8hrqdh4384idi2v";
}
