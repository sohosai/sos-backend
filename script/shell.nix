{ pkgs ? import ../nix/pkgs.nix }:
let
  crate2nix = import ../nix/crate2nix.nix { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    crate2nix
    findutils
    gitMinimal
    gnused
    jq
    nodejs
    yq-go
  ];
}
