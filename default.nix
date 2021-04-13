{ pkgs ? import ./nix/pkgs.nix
, runTests ? true
}:
let
  # sos21-database (that uses sqlx) needs cargo during compilation
  cargoMetadataNoDeps = pkgs.writeShellScript "cargo-wrapped" ''
    [ "$1" != "metadata" ] && exit 1
    shift
    ${pkgs.cargo}/bin/cargo metadata "$@" --no-deps
  '';
  cargoNix = pkgs.callPackage ./Cargo.nix {
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      sos21-database = oldAttrs: {
        CARGO = "${cargoMetadataNoDeps}";
      };
      sos21-api-server = oldAttrs: {
        src = pkgs.symlinkJoin {
          name = "${oldAttrs.crateName}-src";
          paths = [ oldAttrs.src ];
          postBuild = ''
            cp -r ${builtins.path {
              name = "git";
              path = ./.git;
            }} $out/.git
          '';
        };
      };
    };
  };
in
pkgs.lib.mapAttrs
  (_: crate: crate.build.override {
    inherit runTests;
  })
  cargoNix.workspaceMembers
