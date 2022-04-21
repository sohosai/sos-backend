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
    virtualHosts."api.nightly.online.sohosai.com" = {
      enableACME = true;
      forceSSL = true;
      locations."/" = {
        proxyPass = apiServer;
      };
      locations."/file/create" = {
        proxyPass = "${apiServer}/file/create";
        extraConfig = ''
          proxy_request_buffering off;
          client_max_body_size 0;
        '';
      };
    };
  };

  services.sos21-api-server = {
    enable = true;
    port = 3000;
    firebaseProjectId = "sos22-nightly";
    databaseName = "sos21-nightly";
    s3ObjectBucket = "sos21-nightly-objects";
  };
}
