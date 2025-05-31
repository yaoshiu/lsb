{ rustPlatform, ... }:
rustPlatform.buildRustPackage (_: {
  pname = "lsb-core";
  version = "0.1.0";

  src = ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  buildAndTestSubdir = "lsb-core";
})
