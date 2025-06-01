{ rustPlatform, installShellFiles, ... }:
rustPlatform.buildRustPackage (_: {
  pname = "lsb-core";
  version = "0.1.0";

  src = ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  buildAndTestSubdir = "lsb-core";

  nativeBuildInputs = [ installShellFiles ];

  postInstall = ''
    for shell in bash fish zsh; do
      installShellCompletion --cmd lsb-core --''${shell} <($out/bin/lsb-core completion $shell)
    done
  '';
})
