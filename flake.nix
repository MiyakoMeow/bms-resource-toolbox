{
  description = "BMS Resource Toolbox - Nix development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          # Rust
          (final: prev: {
            rust-bin = rust-overlay.lib.mkRustBin { distRoot = "https://rsproxy.cn/dist"; } prev;
          })
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # System dependencies for unrar and other libraries
        buildInputs = with pkgs; [
          # Rust toolchain from rust-overlay
          pkgs.rust-bin.stable.latest.complete

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

          # Essential GUI libraries for iced applications
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi

          # Wayland libraries
          wayland
          wayland-protocols
          libxkbcommon
          xkeyboard_config

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

          # CJK-capable fonts so GUI can render Chinese by default
          noto-fonts
          noto-fonts-cjk-sans
          noto-fonts-cjk-serif
          noto-fonts-emoji
          source-han-sans
          source-han-serif
          wqy_zenhei
          wqy_microhei
        ];

        # Native build inputs for linking
        nativeBuildInputs = with pkgs; [
          cmake
          makeWrapper

          # X Virtual Framebuffer for headless GUI testing
          xvfb-run
        ];

        # Environment variables
        # Prepare a Fontconfig configuration that includes fonts from Nix store
        fontsDirs = builtins.map (f: "${f}/share/fonts") [
          pkgs.noto-fonts
          pkgs.noto-fonts-cjk-sans
          pkgs.noto-fonts-cjk-serif
          pkgs.noto-fonts-emoji
          pkgs.source-han-sans
          pkgs.source-han-serif
          pkgs.wqy_zenhei
          pkgs.wqy_microhei
        ];
        fontsConf = pkgs.makeFontsConf { fontDirectories = fontsDirs; };

        shellHook = ''
          echo "🚀 BMS Resource Toolbox Development Environment"
          echo "Rust version: $(rustc --version)"
          echo "Cargo version: $(cargo --version)"
          export RUST_BACKTRACE=1
          export CARGO_BUILD_JOBS=$NIX_BUILD_CORES
          export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
          export LD_LIBRARY_PATH="${pkgs.llvmPackages.libclang.lib}/lib:$LD_LIBRARY_PATH"

          # Essential GUI library paths including Wayland
          export LD_LIBRARY_PATH="${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:${pkgs.libGL}/lib:${pkgs.fontconfig}/lib:${pkgs.freetype}/lib:$LD_LIBRARY_PATH"
          export XKB_CONFIG_ROOT="${pkgs.xkeyboard_config}/share/X11/xkb"

          # Ensure fontconfig sees Nix-provided fonts
          export FONTCONFIG_FILE="${fontsConf}"

          echo "Ready to run GUI application."
          echo "Run: cargo run"
        '';
      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs shellHook;

          # Environment variables for Rust development
          env = {
            RUST_SRC_PATH = "${pkgs.rust-bin.stable.latest.complete}/lib/rustlib/src/rust/library";
          };
        };

        # Formatter for nix files
        formatter = pkgs.nixfmt-tree;
      }
    );
}
