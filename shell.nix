let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [(import sources.nixpkgs-mozilla)]; };
  unstable = import sources.nixpkgs-unstable {};

  rust = pkgs.rustChannelOfTargets "stable" null [];

  nestalgic = import ./default.nix {};
in

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustc
    cargo

    rustfmt
    unstable.rust-analyzer

    # Library dependencies:
    pkgconfig
  ];

  buildInputs = with pkgs; [
    xlibs.libX11

    graphviz
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
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
  '';

}
