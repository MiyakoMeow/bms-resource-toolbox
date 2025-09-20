{
  description = "BMS Resource Toolbox - Nix development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # System dependencies for unrar and other libraries
        buildInputs = with pkgs; [
          # Core build tools
          rustc
          cargo
          rust-analyzer
          rustfmt
          clippy

          # C/C++ build tools for native dependencies
          clang
          llvmPackages.libclang
          pkg-config

          # Archive utilities
          libarchive
          p7zip

          # System libraries for unrar
          gcc
          gcc-unwrapped

          # GUI/X11 libraries for iced applications
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libXext
          xorg.libXfixes
          xorg.libXrender
          xorg.libXinerama
          xorg.libXft
          xorg.libXcomposite
          xorg.libXdamage
          xorg.libXtst
          xorg.libXScrnSaver

          # Wayland libraries
          wayland
          wayland-protocols
          libxkbcommon

          # Additional libraries that might be needed
          openssl
          sqlite
          libiconv

          # Font libraries
          fontconfig
          freetype
          libGL
          libglvnd
          mesa
          libdrm
        ];

        # Native build inputs for linking
        nativeBuildInputs = with pkgs; [
          cmake
          makeWrapper

          # X Virtual Framebuffer for headless GUI testing
          xvfb-run
        ];

        # Environment variables
        shellHook = ''
          echo "🚀 BMS Resource Toolbox Development Environment"
          echo "Rust version: $(rustc --version)"
          echo "Cargo version: $(cargo --version)"
          export RUST_BACKTRACE=1
          export CARGO_BUILD_JOBS=$NIX_BUILD_CORES
          export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
          export LD_LIBRARY_PATH="${pkgs.llvmPackages.libclang.lib}/lib:$LD_LIBRARY_PATH"

          # GUI application library paths for Hyprland/Wayland
          export LD_LIBRARY_PATH="${pkgs.libGL}/lib:${pkgs.fontconfig}/lib:${pkgs.freetype}/lib:$LD_LIBRARY_PATH"

          # Don't override existing display environment variables in Hyprland
          # Let the native Hyprland/Wayland environment take precedence

          echo "GUI libraries loaded. Ready for Hyprland/Wayland environment."
          echo "Run: cargo run --bin bms-resource-toolbox-gui"
        '';
      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs shellHook;

          # Environment variables for Rust development
          env = {
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        };

        # Formatter for nix files
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
