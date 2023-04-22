{ rustPlatform, fetchFromGitHub, openssl, pkg-config, lib }:
rustPlatform.buildRustPackage rec {
  pname = "sqlx-cli";
  version = "0.6.3";
  src = builtins.fetchTarball {
    url = "https://github.com/launchbadge/sqlx/archive/v0.6.3.tar.gz";
    sha256 = "11j8vjb9dz551894379gikw6blsaqdchkx19gl62rzbkfcfrpcmc";
  };
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
  cargoBuildFlags = [ "--manifest-path=sqlx-cli/Cargo.toml" ];
  cargoSha256 = "WRVrxBQytxogdaiG6KTmSTaSzN1gh9cltYkVEf4wUUI=";
  doCheck = false;
  meta = with lib; {
    description = "Command-line utility for SQLx, the Rust SQL toolkit.";
    license = licenses.mit;
    platforms = platforms.all;
  };
}
