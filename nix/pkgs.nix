let
  nixpkgs = builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/refs/tags/22.11.tar.gz";
    sha256 = "11w3wn2yjhaa5pv20gbfbirvjq6i3m7pqrq2msf0g7cv44vijwgw";
  };
  moz_overlay = builtins.fetchTarball {
    url = "https://github.com/mozilla/nixpkgs-mozilla/archive/78e723925daf5c9e8d0a1837ec27059e61649cb6.tar.gz";
    sha256 = "0k3jxk21s28jsfpmqv39vyhfz2srfm81kp4xnpzgsbjn77rhwn03";
  };
in
import nixpkgs {
  overlays = [
    (import moz_overlay)
    (import ./overlay.nix)
  ];
}
