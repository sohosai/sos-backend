{ rustChannelOf, makeRustPlatform }:
let
  channel = rustChannelOf {
    channel = "1.69.0";
    sha256 = "eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";
  };
in
makeRustPlatform {
  rustc = channel.rust;
  cargo = channel.rust;
}
