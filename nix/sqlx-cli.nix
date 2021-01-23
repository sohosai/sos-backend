{ rustPlatform, fetchFromGitHub, openssl, pkg-config, lib }:
rustPlatform.buildRustPackage rec {
  pname = "sqlx-cli";
  version = "0.2.0";
  src = fetchFromGitHub {
    owner = "launchbadge";
    repo = "sqlx";
    rev = "v0.4.2";
    sha256 = "1q6p4qly9qjn333nj72sar6nbyclfdw9i9l6rnimswylj0rr9n27";
  };
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
  cargoBuildFlags = [ "--manifest-path=sqlx-cli/Cargo.toml" ];
  cargoSha256 = "1cxihnk82anyvryvl0d95323ppjn4v27ykz2c1sflmva7k34prhd";
  doCheck = false;
  meta = with lib; {
    description = "Command-line utility for SQLx, the Rust SQL toolkit.";
    license = licenses.mit;
    platforms = platforms.all;
  };
}
