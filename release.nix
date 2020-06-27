let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs {};

in

{
  nestalgic-linux = import ./default.nix {
    target = "x86_64-unknown-linux-gnu";
    targetPkgsOverride = pkgs;
  };

  nestalgic-windows = import ./default.nix {
    crossSystem = pkgs.lib.systems.examples.mingwW64;
    target = "x86_64-pc-windows-gnu";
    targetPkgsOverride = import sources.nixpkgs {
      crossSystem = pkgs.lib.systems.examples.mingwW64;
    };
  };
}
