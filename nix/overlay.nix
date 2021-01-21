self: super: {
  rustPlatform = import ./rust.nix { pkgs = self; };
}
