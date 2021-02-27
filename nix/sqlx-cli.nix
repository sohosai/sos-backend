{ rustPlatform, fetchFromGitHub, openssl, pkg-config, lib }:
rustPlatform.buildRustPackage rec {
  pname = "sqlx-cli";
  version = "0.5.1";
  src = builtins.fetchTarball {
    url = "https://github.com/launchbadge/sqlx/archive/v0.5.1.tar.gz";
    sha256 = "02phkrcjszs6gdq0yva9fv9f8c4bda0vp9alml7kr5fj65gns8mh";
  };
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
  cargoBuildFlags = [ "--manifest-path=sqlx-cli/Cargo.toml" ];
  cargoSha256 = "1899jwqvdrsdhncg107k0i3w8l496gz0d73zdj2mxnj2lfmpfq0s";
  doCheck = false;
  meta = with lib; {
    description = "Command-line utility for SQLx, the Rust SQL toolkit.";
    license = licenses.mit;
    platforms = platforms.all;
  };
}
