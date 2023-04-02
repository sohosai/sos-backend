{ config, lib, pkgs, ... }:

with lib;
let
  cfg = config.services.sos21-api-server;
  sos21-backend = import ../../. { };
  inherit (sos21-backend) sos21-api-server sos21-run-migrations;

  makeKeyLoadScript = { name, keys, envFile, user, group }:
    let
      exports = concatStringsSep "\n" (mapAttrsToList
        (keyName: keyPath: ''
          export ${keyName}=$(cat '${toString keyPath}')
        '')
        keys);
    in
    pkgs.writeShellScript name ''
      touch '${envFile}'
      chmod 600 '${envFile}'
      cat << EOS > '${envFile}'
      ${exports}
      EOS
      chown '${user}:${group}' '${envFile}'
    '';
in
{

  options = {

    services.sos21-api-server = {

      enable = mkEnableOption "sos21-api-server";

      port = mkOption {
        type = types.int;
      };

      administratorEmail = mkOption {
        type = types.str;
      };

      projectCreationPeriods = mkOption {
        type = types.attrsOf types.str;
        default = { };
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

    users.users.sos23 = {
      name = "sos23";
      group = "sos23";
    };
    users.groups.sos21 = {
      name = "sos23";
    };

    systemd.services.sos21-run-migrations =
      let
        envFile = "/var/tmp/sos21-run-migrations-keys";
        preStart = makeKeyLoadScript {
          name = "sos21-run-migrations-pre-start";
          user = config.users.users.sos23.name;
          group = config.users.users.sos23.group;
          inherit envFile;
          keys = {
            DB_USERNAME = cfg.databaseUsernameFile;
            DB_PASSWORD = cfg.databasePasswordFile;
          };
        };
      in
      {
        after = [ "postgresql.service" ];
        wantedBy = [ "multi-user.target" ];
        serviceConfig = {
          Type = "oneshot";
          ExecStartPre = "+${preStart}";
          User = config.users.users.sos23.name;
          Group = config.users.users.sos23.group;
        };
        postStart = "rm -f '${envFile}'";
        script = ''
          source '${envFile}'
          export SOS21_RUN_MIGRATIONS_POSTGRES_URI="postgres://$DB_USERNAME:$DB_PASSWORD@${cfg.databaseHost}:${toString cfg.databasePort}/${cfg.databaseName}"
          export PGPASSWORD=$DB_PASSWORD

          HAS_DB=$(
            ${pkgs.postgresql_13}/bin/psql postgres -h '${cfg.databaseHost}' -U "$DB_USERNAME" \
              -tA -c "SELECT true FROM pg_database WHERE datname = '${cfg.databaseName}'"
          )
          if [ "$HAS_DB" != "t" ]; then
             ${pkgs.postgresql_13}/bin/createdb -h '${cfg.databaseHost}' -U "$DB_USERNAME" '${cfg.databaseName}'
          fi

          ${sos21-run-migrations}/bin/sos21-run-migrations --wait
        '';
      };

    systemd.services.sos21-create-bucket =
      let
        envFile = "/var/tmp/sos21-create-bucket-keys";
        preStart = makeKeyLoadScript {
          name = "sos21-create-bucket-pre-start";
          user = config.users.users.sos23.name;
          group = config.users.users.sos23.group;
          inherit envFile;
          keys = {
            ACCESS_KEY = cfg.s3AccessKeyFile;
            ACCESS_SECRET = cfg.s3AccessSecretFile;
          };
        };
      in
      {
        after = [ "minio.service" ];
        wantedBy = [ "multi-user.target" ];
        serviceConfig = {
          Type = "oneshot";
          ExecStartPre = "+${preStart}";
          User = config.users.users.sos23.name;
          Group = config.users.users.sos23.group;
        };
        postStart = "rm -f '${envFile}'";
        script = ''
          source '${envFile}'
          endpoint=${cfg.s3Endpoint}
          scheme=''${endpoint%://*}
          endpoint=''${endpoint#*://}
          export MC_HOST_minio=$scheme://$ACCESS_KEY:$ACCESS_SECRET@$endpoint
          # TODO: run without config dir
          mkdir -p /var/tmp/.mc
          ${pkgs.minio-client}/bin/mc -C /var/tmp/.mc mb -p 'minio/${cfg.s3ObjectBucket}'
        '';
      };

    systemd.services.sos21-api-server =
      let
        envFile = "/var/tmp/sos21-api-server-keys";
        preStart = makeKeyLoadScript {
          name = "sos21-api-server-pre-start";
          user = config.users.users.sos23.name;
          group = config.users.users.sos23.group;
          inherit envFile;
          keys = {
            DB_USERNAME = cfg.databaseUsernameFile;
            DB_PASSWORD = cfg.databasePasswordFile;
            SOS21_API_SERVER_S3_ACCESS_KEY = cfg.s3AccessKeyFile;
            SOS21_API_SERVER_S3_ACCESS_SECRET = cfg.s3AccessSecretFile;
          };
        };
      in
      {
        wantedBy = [ "multi-user.target" ];
        after = [
          "sos21-run-migrations.service"
          "sos21-create-bucket.service"
          "network-online.target"
        ];
        serviceConfig = {
          ExecStartPre = "+${preStart}";
          User = config.users.users.sos23.name;
          Group = config.users.users.sos23.group;
          AmbientCapabilities = "CAP_NET_BIND_SERVICE";
        };
        postStop = "rm -f '${envFile}'";
        environment = filterAttrs (_: v: v != null)
          {
            SOS21_API_SERVER_JWT_AUDIENCE = cfg.firebaseProjectId;
            SOS21_API_SERVER_JWT_ISSUER = "https://securetoken.google.com/${cfg.firebaseProjectId}";
            SOS21_API_SERVER_JWT_KEYS_URL = "https://www.googleapis.com/robot/v1/metadata/jwk/securetoken@system.gserviceaccount.com";
            SOS21_API_SERVER_S3_REGION = cfg.s3Region;
            SOS21_API_SERVER_S3_ENDPOINT = cfg.s3Endpoint;
            SOS21_API_SERVER_S3_OBJECT_BUCKET = cfg.s3ObjectBucket;
            SOS21_API_SERVER_ADMINISTRATOR_EMAIL = cfg.administratorEmail;
            SOS21_API_SERVER_BIND = "0.0.0.0:${toString cfg.port}";
          } // mapAttrs' (n: v: nameValuePair "SOS21_API_SERVER_PROJECT_CREATION_PERIOD_${n}" v) cfg.projectCreationPeriods;
        script = ''
          source '${envFile}'
          rm -f '${envFile}'
          export SOS21_API_SERVER_POSTGRES_URI="postgres://$DB_USERNAME:$DB_PASSWORD@${cfg.databaseHost}:${toString cfg.databasePort}/${cfg.databaseName}"
          ${sos21-api-server}/bin/sos21-api-server
        '';
      };

  };

}
