{ config, pkgs, ... }:
let
  apiServer = "http://localhost:${toString config.services.sos21-api-server.port}";
in
{
  imports = [
    ./sos21-api-server.nix
    ./nginx.nix
    ./acme.nix
    ./staging.nix
  ];

  services.nginx = {
    virtualHosts."api.beta.online.sohosai.com" = {
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
    firebaseProjectId = "sos21-beta";
    databaseName = "sos21-beta";
    s3ObjectBucket = "sos21-beta-objects";
  };
}