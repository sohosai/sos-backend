{ pkgs ? import ./pkgs.nix }:
let channel = pkgs.rustChannels.stable;
in
pkgs.makeRustPlatform {
  rustc = channel.rust;
  cargo = channel.rust;
}
