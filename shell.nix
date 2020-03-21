with import <nixpkgs> {};

stdenv.mkDerivation rec {
  name = "rust-env";
  env = buildEnv { name = name; paths = buildInputs; };

  buildInputs = [
    latest.rustChannels.nightly.rust
    openssl
    fish
    pkg-config
    protobuf
  ];

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";
}
