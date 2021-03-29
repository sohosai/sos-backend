{ pkgs ? import ./nix/pkgs.nix }:
let
  rustPlatform = pkgs.callPackage ./nix/rustPlatform.nix { };
in
rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.3.0";
  src = ./.;

  cargoSha256 = "1nq8pyl3xsri74q4ify74cwg2v11f2ifpsdsx29bldz3v1nm73kb";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
