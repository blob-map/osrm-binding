{
  description = "Development environment for osrm-binding";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          setupTestData = pkgs.writeShellApplication {
            name = "setup-test-data";
            runtimeInputs = with pkgs; [
              coreutils
              curl
              findutils
              git
              gnugrep
            ];
            text = ''
              exec ${./scripts/setup-test-data.sh} "$@"
            '';
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Rust development tools
              cargo
              clippy
              rust-analyzer
              rustc
              rustfmt
              setupTestData

              # C++/CMake toolchain used by build.rs to compile OSRM
              cmake
              git
              pkg-config
              stdenv.cc

              # OSRM and reqwest native dependencies
              # OSRM v6.0.0 uses Boost APIs removed from newer releases.
              boost181
              bzip2
              cacert
              expat
              fmt
              libxml2
              libzip
              lua5_4
              openssl
              tbb
              zlib
            ];

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt-tree);
    };
}
