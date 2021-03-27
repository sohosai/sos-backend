{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.1";
  src = ./.;

  cargoSha256 = "020252zd06g1vl24zh0z86xd50x8cknyrw3ibg5bbmh1z0x6qv7f";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
