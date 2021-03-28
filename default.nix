{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.1";
  src = ./.;

  cargoSha256 = "03f69nvcx63kcxdr1cvcwn48fgjbwvs3hxh8jdj0mf2a0khxh6r7";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
