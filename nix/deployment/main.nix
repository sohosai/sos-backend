{ config, pkgs, ... }:
let
  apiServer = "http://localhost:${toString config.services.sos21-api-server.port}";
in
{
  imports = [
    ./sos21-api-server.nix
    ./nginx.nix
    ./acme.nix
    ./production.nix
  ];

  services.nginx = {
    clientMaxBodySize = "2m";

    virtualHosts."api.online.sohosai.com" = {
      enableACME = true;
      forceSSL = true;
      locations."/" = {
        proxyPass = apiServer;
      };
      locations."/file/create" = {
        proxyPass = "${apiServer}/file/create";
        extraConfig = ''
          client_max_body_size 0;
        '';
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