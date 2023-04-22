{ rustPlatform, fetchFromGitHub, openssl, pkg-config, lib }:
rustPlatform.buildRustPackage rec {
  pname = "sqlx-cli";
  version = "0.5.8";
  src = builtins.fetchTarball {
    url = "https://github.com/launchbadge/sqlx/archive/v0.5.10.tar.gz";
    sha256 = "0d0yv9hvd73cl5h4fc061ghhvk3c8kgii4sjxwngq2zl318jw4iq";
  };
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
  cargoBuildFlags = [ "--manifest-path=sqlx-cli/Cargo.toml" ];
  cargoSha256 = "eeZZoQ9gwN0A+MeCxIaFWEy4y5ZnEeakwgcO8mWSVXI=";
  doCheck = false;
  meta = with lib; {
    description = "Command-line utility for SQLx, the Rust SQL toolkit.";
    license = licenses.mit;
    platforms = platforms.all;
  };
}
