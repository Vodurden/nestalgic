let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs {};

  muslPkgs = (import sources.nixpkgs {
    config = {
      # Fix openssh infinite recursion. Can be removed when the fix lands in the stable nixpkgs channels
      #
      # See: https://github.com/NixOS/nixpkgs/commit/59616b291d60886606ca300c20107722f284cdf7
      packageOverrides = pkgs: {
        openssh = pkgs.openssh.override { withFIDO = false; };
      };
    };
  }).pkgsMusl;

in

{
  nestalgic-linux = import ./default.nix {
    target = "x86_64-unknown-linux-musl";
    targetPkgsOverride = muslPkgs;
  };

  nestalgic-windows = import ./default.nix {
    crossSystem = pkgs.lib.systems.examples.mingwW64;
    target = "x86_64-pc-windows-gnu";
    targetPkgsOverride = import sources.nixpkgs {
      crossSystem = pkgs.lib.systems.examples.mingwW64;
    };
  };
}
