{
  description = "Embassy STM32F103 cross-compilation using Nix and Fenix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        config = {allowUnfree = true;};
      };

      # Rust toolchain with target support
      rustToolchain = fenix.packages.${system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
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
        stm32cubemx
        (pkgs.python3.withPackages (ps: with ps; [pyocd]))
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
          export DEFMT_LOG=trace
          echo "🔧 Target: $CARGO_TARGET"
        '';
      };
    });
}
