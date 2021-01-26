{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "0iqi2qz4mz72grm5lv0k945rlg0rwvm1c8z452363vwn2qnv6ac4";
}
