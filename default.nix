{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "0clgz2h4w3nysfhvd5cl0n82lyzydnhi8s4n3vwp7vm4qgznccsf";
}
