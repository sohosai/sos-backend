{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
rustPlatform.buildRustPackage {
  pname = "sos21-api-server";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "157k9lgd88ykv28fxiizd6x9kj4m2c97k31r96wa189g8kl4kq6b";
}
