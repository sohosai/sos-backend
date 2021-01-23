self: super: {
  rustPlatform = super.callPackage ./rustPlatform.nix { };
  sqlx-cli = super.callPackage ./sqlx-cli.nix { };
}
