{ rustChannelOf, makeRustPlatform }:
let
  channel = rustChannelOf {
    channel = "1.52.1";
    sha256 = "157iggldvb9lcr45zsld6af63yp370f3hyswcb0zwjranrg69r79";
  };
in
makeRustPlatform {
  rustc = channel.rust;
  cargo = channel.rust;
}
