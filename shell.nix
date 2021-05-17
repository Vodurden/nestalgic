let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [(import sources.nixpkgs-mozilla)]; };
  unstable = import sources.nixpkgs-unstable {};

  # rust = pkgs.rustChannelOfTargets "stable" null [];
  rustChannel = pkgs.rustChannelOf { channel = "stable"; };

  nestalgic = import ./default.nix {};
in

pkgs.mkShell {
  nativeBuildInputs = [
    rustChannel.rust

    pkgs.rustfmt
    # unstable.rust-analyzer

    # Library dependencies:
    pkgs.pkgconfig
  ];

  buildInputs = [
    pkgs.xlibs.libX11

    pkgs.graphviz
  ];

  APPEND_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    libGL
    vulkan-loader
    xorg.libX11
    xlibs.libXcursor
    xlibs.libXi
    xlibs.libXrandr
  ];

  shellHook = ''
    export RUST_SRC_PATH="${rustChannel.rust-src}/lib/rustlib/src/rust/src"
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
  '';

}
