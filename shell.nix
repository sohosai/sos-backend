{ pkgs ? import ./nix/pkgs.nix }:
let
  sos21-backend = import ./. { inherit pkgs; };
  sqlx-cli = pkgs.callPackage ./nix/sqlx-cli.nix { };
  crate2nix = builtins.fetchTarball {
    url = "https://github.com/kolloch/crate2nix/tarball/0.9.0";
    sha256 = "0lxi9zl5mzzpz7gsa7pbqag163hv84xd6diyj0f0y5whf1mk07vl";
  };
in
sos21-backend.overrideAttrs (oldAttrs: {
  nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
    sqlx-cli
    crate2nix
  ];
})
