{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
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
        };
      }
    );
}
