{ config, lib, pkgs, ... }:

with lib;
let
  cfg = config.services.sos21-api-server;
  sos21-backend = import ../../. { };
  inherit (sos21-backend) sos21-api-server sos21-run-migrations;
in
{

  options = {

    services.sos21-api-server = {

      enable = mkEnableOption "sos21-api-server";

      port = mkOption {
        type = types.int;
      };

      firebaseProjectId = mkOption {
        type = types.str;
      };

      databaseHost = mkOption {
        type = types.str;
      };

      databaseUsernameFile = mkOption {
        type = types.path;
      };

      databasePasswordFile = mkOption {
        type = types.path;
      };

      databasePort = mkOption {
        type = types.int;
      };

      databaseName = mkOption {
        type = types.str;
      };

      s3AccessKeyFile = mkOption {
        type = types.path;
      };

      s3AccessSecretFile = mkOption {
        type = types.path;
      };

      s3Region = mkOption {
        type = types.str;
        default = "";
      };

      s3Endpoint = mkOption {
        type = types.str;
      };

      s3ObjectBucket = mkOption {
        type = types.str;
      };
    };

  };

  config = mkIf cfg.enable {

    systemd.services.sos21-run-migrations = {
      after = [ "postgresql.service" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "oneshot";
      };
      script = ''
        DB_USERNAME=$(cat ${toString cfg.databaseUsernameFile})
        DB_PASSWORD=$(cat ${toString cfg.databasePasswordFile})
        export SOS21_RUN_MIGRATIONS_POSTGRES_URI="postgres://$DB_USERNAME:$DB_PASSWORD@${cfg.databaseHost}:${toString cfg.databasePort}/${cfg.databaseName}"
        export PGPASSWORD=$DB_PASSWORD

        HAS_DB=$(
          ${pkgs.postgresql_13}/bin/psql postgres -h '${cfg.databaseHost}' -U "$DB_USERNAME" \
            -tA -c "SELECT true FROM pg_database WHERE datname = '${cfg.databaseName}'"
        )
        if [ "$HAS_DB" != "t" ]; then
           ${pkgs.postgresql_13}/bin/createdb -h '${cfg.databaseHost}' -U "$DB_USERNAME" '${cfg.databaseName}'
        fi

        ${sos21-run-migrations}/bin/sos21-run-migrations
      '';
    };

    systemd.services.sos21-create-bucket = {
      after = [ "minio.service" ];
      wantedBy = [ "multi-user.target" ];
      path = [ pkgs.getent ];  # needed by mc
      serviceConfig = {
        Type = "oneshot";
      };
      script = ''
        ACCESS_KEY=$(cat ${toString cfg.s3AccessKeyFile})
        ACCESS_SECRET=$(cat ${toString cfg.s3AccessSecretFile})
        ${pkgs.minio-client}/bin/mc alias set minio '${cfg.s3Endpoint}' $ACCESS_KEY $ACCESS_SECRET
        ${pkgs.minio-client}/bin/mc mb -p 'minio/${cfg.s3ObjectBucket}'
      '';
    };

    systemd.services.sos21-api-server = {
      wantedBy = [ "multi-user.target" ];
      after = [
        "sos21-run-migrations.service"
        "sos21-create-bucket.service"
        "network-online.target"
      ];
      environment = {
        SOS21_API_SERVER_JWT_AUDIENCE = cfg.firebaseProjectId;
        SOS21_API_SERVER_JWT_ISSUER = "https://securetoken.google.com/${cfg.firebaseProjectId}";
        SOS21_API_SERVER_JWT_KEYS_URL = "https://www.googleapis.com/robot/v1/metadata/jwk/securetoken@system.gserviceaccount.com";
        SOS21_API_SERVER_S3_REGION = cfg.s3Region;
        SOS21_API_SERVER_S3_ENDPOINT = cfg.s3Endpoint;
        SOS21_API_SERVER_S3_OBJECT_BUCKET = cfg.s3ObjectBucket;
        SOS21_API_SERVER_BIND = "0.0.0.0:${toString cfg.port}";
      };
      script = ''
        DB_USERNAME=$(cat ${toString cfg.databaseUsernameFile})
        DB_PASSWORD=$(cat ${toString cfg.databasePasswordFile})
        export SOS21_API_SERVER_POSTGRES_URI="postgres://$DB_USERNAME:$DB_PASSWORD@${cfg.databaseHost}:${toString cfg.databasePort}/${cfg.databaseName}"
        export SOS21_API_SERVER_S3_ACCESS_KEY=$(cat ${toString cfg.s3AccessKeyFile})
        export SOS21_API_SERVER_S3_ACCESS_SECRET=$(cat ${toString cfg.s3AccessSecretFile})
        ${sos21-api-server}/bin/sos21-api-server
      '';
    };

  };

}
