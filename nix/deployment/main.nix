{ config, pkgs, ... }:
{
  imports = [
    ./sos21-api-server.nix
    ./nginx.nix
    ./acme.nix
    ./production.nix
  ];

  services.nginx = {
    virtualHosts."api.online.sohosai.com" = {
      enableACME = true;
      forceSSL = true;
      locations."/" = {
        proxyPass = "http://localhost:${toString config.services.sos21-api-server.port}/";
      };
    };
  };

  services.sos21-api-server = {
    enable = true;
    port = 3000;
    firebaseProjectId = "sos21-main";
    databaseName = "sos21-main";
    s3ObjectBucket = "sos21-main-objects";
  };
}
