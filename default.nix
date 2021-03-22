{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.1";
  src = ./.;

  cargoSha256 = "1953jalyzsl7g729d8h4vrq2b1qvdkfzyxqx03ygwf4zg6x4fbsj";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
