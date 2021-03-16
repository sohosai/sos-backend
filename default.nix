{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.1.0";
  src = ./.;

  cargoSha256 = "1q8vb047z9dmxmajvvlbr3anwm87h1s0jgay3yvmjkxvb2zrksb3";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
