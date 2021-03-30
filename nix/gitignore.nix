{ lib }:
let
  gitignoreSrc = builtins.fetchTarball {
    url = "https://github.com/hercules-ci/gitignore.nix/archive/211907489e9f198594c0eb0ca9256a1949c9d412.tar.gz";
    sha256 = "06j7wpvj54khw0z10fjyi31kpafkr6hi1k0di13k1xp8kywvfyx8";
  };
in
import gitignoreSrc { inherit lib; }
