{ pkgs ? import ./nix/pkgs.nix }:

pkgs.rustPlatform.buildRustPackage {
  pname = "sos21-backend";
  version = "0.2.0";
  src = ./.;

  cargoSha256 = "1dyqh7larb4rm4qc0swp6y4ffrhhv6lzx6nlrmbpkvq5lqb8d18q";

  nativeBuildInputs = with pkgs; [
    zlib
  ];
}
