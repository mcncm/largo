{
  description = "Xargo TeX build system";

  inputs = {
    # Rust toolchain
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rustPlatform = pkgs.makeRustPlatform {
          inherit (fenix.packages.${system}.minimal) cargo rustc;
        };

        darwinBuildInputs = with pkgs; [ pkgconfig libiconv ];

        buildInputs = [ ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwinBuildInputs;

        binWithFeatures = { buildFeatures ? [ ] }:
          rustPlatform.buildRustPackage {
            pname = "xargo";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            inherit buildFeatures;
            doCheck = false; # Ignore tests (for now)
            inherit buildInputs;
          };
      in {
        packages.default = (binWithFeatures { });

        packages.withFeatures = buildFeatures:
          binWithFeatures { inherit buildFeatures; };
      });
}
