{
  description = "Embassy STM32F103 cross-compilation using Nix and Fenix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Rust toolchain with target support
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
        };

        # Tools for embedded dev
        extraTools = with pkgs; [
          rustToolchain
          llvm
          pkg-config
          gdb
          stlink
          openocd
          probe-rs
          cargo-binutils
          dfu-util
          gcc-arm-embedded
          (pkgs.python3.withPackages (ps: with ps; [ pyocd ]))
        ];
      in {
        devShells.default = pkgs.mkShell {
          name = "stm32-dev";
          packages = extraTools;

          # Enable cargo cross-compilation
          CARGO_TARGET_THUMBV7M_NONE_EABI_LINKER = "arm-none-eabi-ld";
          RUST_TARGET_PATH = ".";

          shellHook = ''
            export CARGO_TARGET=thumbv7m-none-eabi
            echo "🔧 Target: $CARGO_TARGET"
          '';
        };
      });
}
