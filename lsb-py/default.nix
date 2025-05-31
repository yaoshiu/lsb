{
  buildPythonPackage,
  rustPlatform,
}:
buildPythonPackage rec {
  pname = "lsb-py";
  version = "0.1.0";

  src = ./..;

  pyproject = true;

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ../Cargo.lock;
  };

  buildAndTestSubdir = "lsb-py";

  nativeBuildInputs = with rustPlatform; [
    cargoSetupHook
    maturinBuildHook
  ];
}
