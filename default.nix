{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "0w3js5qkyap80q4arff2246xmrx2f4qzvqh44smarkl0yryxg9ls";
}
