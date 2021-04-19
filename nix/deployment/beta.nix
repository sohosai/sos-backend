{ config, pkgs, ... }:
{
  imports = [
    ./sos21-api-server.nix
    ./staging.nix
  ];

  networking.firewall.allowedTCPPorts = [ 80 ];

  services.sos21-api-server = {
    enable = true;
    port = 80;
    firebaseProjectId = "sos21-beta";
    databaseName = "sos21-beta";
    s3ObjectBucket = "sos21-beta-objects";
  };
}
