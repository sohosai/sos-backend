self: super:
let
  rustPlatform = super.callPackage ./rustPlatform.nix { };
in
{
  inherit rustPlatform;
  inherit (rustPlatform.rust) rustc cargo;
}
