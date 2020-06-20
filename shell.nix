let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [(import sources.nixpkgs-mozilla)]; };

  rust = pkgs.rustChannelOfTargets "stable" null [];

  nestalgic = import ./default.nix {};
in

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustc
    cargo

    pkgconfig
  ];

  buildInputs = with pkgs; [
    xlibs.libX11
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
