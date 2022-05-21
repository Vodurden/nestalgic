{
  description = "A toy NES emulator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixgl.url = "github:guibou/nixGL/";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, nixgl }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay) nixgl.overlay];
        };

        rust = (pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer-preview"
          ];
        });
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = [
            rust
            pkgs.pkgconfig
          ];

          buildInputs = [
            pkgs.xorg.libX11
            pkgs.xlibs.libX11
            pkgs.wayland
            pkgs.nixgl.nixVulkanIntel
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.xlibs.libX11
            pkgs.xlibs.libXcursor
            pkgs.xlibs.libXrandr
            pkgs.xlibs.libXi
            pkgs.libxkbcommon
            pkgs.vulkan-loader
            pkgs.wayland
            pkgs.xwayland
          ];

          shellHook = ''
            export VK_LAYER_PATH=$(nixVulkanIntel printenv VK_LAYER_PATH)
            export VK_ICD_FILENAMES=$(nixVulkanIntel printenv VK_ICD_FILENAMES)
            export LD_LIBRARY_PATH=$(nixVulkanIntel printenv LD_LIBRARY_PATH):$LD_LIBRARY_PATH
          '';
        };
      }
    );
}
