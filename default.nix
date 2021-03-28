{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.1";
  src = ./.;

  cargoSha256 = "14kfi0prc36cqmlawn22jc9wmmwyqi7mykbbdamkqaykzg7fr4vk";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
