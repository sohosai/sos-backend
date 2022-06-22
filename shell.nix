{ pkgs ? import ./nix/pkgs.nix }:
let
  sqlx-cli = pkgs.callPackage ./nix/sqlx-cli.nix { };
  crate2nix = import ./nix/crate2nix.nix { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    zlib
    rustc
    cargo
    sqlx-cli
    crate2nix
    openssl
    pkgconfig
  ];

  # Do not search $HOME/.cargo/bin for subcommands (rust-lang/cargo#6507)
  CARGO_HOME = toString ./.cargo;
}
