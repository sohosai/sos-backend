{ rustChannels, makeRustPlatform }:
let channel = rustChannels.stable;
in
makeRustPlatform {
  rustc = channel.rust;
  cargo = channel.rust;
}
