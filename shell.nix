{ pkgs ? import ./nix/pkgs.nix }:
with pkgs;
mkShell {
  nativeBuildInputs = with rustPlatform.rust; [ rustc cargo ];

  RUST_BACKTRACE = 1;
}
