{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.1";
  src = ./.;

  cargoSha256 = "1idrfs6avjm6cx75mx13pa9syjsb5q9xx1ywi4v1rlkgzzffnh5m";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
