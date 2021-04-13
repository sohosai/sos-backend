{ pkgs ? import ./pkgs.nix
, name ? "sos21-backend"
, tag ? null
}:
let
  sos21-backend = import ../. { inherit pkgs; };
  inherit (sos21-backend) sos21-api-server sos21-run-migrations;
in
pkgs.dockerTools.buildImage {
  inherit name tag;
  contents = [
    sos21-api-server
    sos21-run-migrations
    # hyper-rustls under rusoto needs the native CA certificates (rusoto/rusoto#1811)
    pkgs.cacert
  ];
  config = {
    Cmd = [ "/bin/sos21-api-server" ];
  };
}
