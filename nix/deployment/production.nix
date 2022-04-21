{ config, pkgs, ... }:
{
  imports = [
    ./sos21-api-server.nix
  ];

  # TODO: Inject these constants from sos22-backend-infrastructure
  services.sos21-api-server = {
    databaseHost = "192.168.0.11";
    databasePort = 5432;
    databaseUsernameFile = /var/keys/database-username;
    databasePasswordFile = /var/keys/database-password;
    s3Endpoint = "http://192.168.0.12:9000";
    s3AccessKeyFile = /var/keys/minio-access-key;
    s3AccessSecretFile = /var/keys/minio-secret-key;
  };
}
