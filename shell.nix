{ pkgs ? import ./nix/pkgs.nix }:
let
  sqlx-cli = pkgs.callPackage ./nix/sqlx-cli.nix { };
  crate2nix = builtins.fetchTarball {
    url = "https://github.com/kolloch/crate2nix/tarball/0.9.0";
    sha256 = "0lxi9zl5mzzpz7gsa7pbqag163hv84xd6diyj0f0y5whf1mk07vl";
  };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    zlib
    rustc
    cargo
    sqlx-cli
    crate2nix
  ];
}
