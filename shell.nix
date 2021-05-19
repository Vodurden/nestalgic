let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [(import sources.rust-overlay)]; };

  unstable = import sources.nixpkgs-unstable {};

  nestalgic = import ./default.nix {};

  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [
      "rust-src"
    ];
  };
in

pkgs.mkShell {
  nativeBuildInputs = [
    # pkgs.rustfmt
    # unstable.rust-analyzer

    # Library dependencies:
    pkgs.pkgconfig

  ];

  buildInputs = [
    rust
    unstable.rust-analyzer
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

  # export RUST_SRC_PATH="${rustChannel.rust-src}/lib/rustlib/src/rust/src"
  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
  '';

}
