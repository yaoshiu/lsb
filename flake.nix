{
  inputs = {
    fenix = {
      inputs.nixpkgs.follows = "nixpkgs";
    };

    wasm-pack-src = {
      url = "github:yaoshiu/wasm-pack";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      flake-utils,
      wasm-pack-src,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            self.overlay
          ];
        };

        rust = import fenix {
          inherit pkgs system;
        };

        toolchain =
          with rust;
          combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
          ];

        wasm-pack = pkgs.wasm-pack.overrideAttrs (
          final: prev: {
            src = wasm-pack-src;

            cargoDeps = pkgs.rustPlatform.importCargoLock {
              lockFile = final.src + "/Cargo.lock";
              outputHashes = {
                "binary-install-0.4.1" = "sha256-AqNbtTIjhmD9lMN7krjTHvxrbOK1Rty8/Z8OIR+zMCw=";
              };
            };

            carghHash = null;
          }
        );
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            just
            toolchain
            (python3.withPackages (
              ps: with ps; [
                lsb-py
              ]
            ))
            maturin
            wasm-pack
          ];
        };

        packages = {
          lsb-py = pkgs.python3Packages.lsb-py;
        };
      }
    )
    // {
      overlay = final: prev: rec {
        python3 = prev.python3.override {
          packageOverrides = final: prev: {
            lsb-py = final.callPackage ./lsb-py { };
          };
        };

        pythonPackages = python3.pkgs;

        lsb-core = final.callPackage ./lsb-core { };
      };
    };
}
