{ pkgs ? import ../nix/pkgs.nix }:
let
  crate2nix = import ../nix/crate2nix.nix { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    coreutils
    crate2nix
    findutils
    gitMinimal
    gnused
    jq
    nodejs
    yq-go
  ];

  # Do not search $HOME/.cargo/bin for subcommands (rust-lang/cargo#6507)
  CARGO_HOME = toString ../.cargo;
}
