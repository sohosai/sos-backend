{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "1ggjddsf72wwxgw4yk0zi3g3cajw0gr7ng8zh55z5nhfgkrp67zb";
}
