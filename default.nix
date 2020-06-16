{ crossSystem ? null, target ? null }:

let
  sources = import nix/sources.nix;
  hostPkgs = import sources.nixpkgs { overlays = [(import sources.nixpkgs-mozilla)]; };

  # Target packages _don't_ import the mozilla-nixpkgs overlay as it overrides `buildRustPackage` with
  # a variant that doesn't respect the `target` attribute.
  #
  # We need the `target` attribute for windows builds so we can override it with `x86_64-pc-windows-gnu`.
  # Normally we could just take the target triple but `mingwW64`'s triple doesn't match with a supported
  # rust toolchain.
  targetPkgs = import sources.nixpkgs { inherit crossSystem; };

  rustChannelTargets = hostPkgs.lib.remove null [target];
  rustChannel = hostPkgs.rustChannelOfTargets "stable" null rustChannelTargets;

  rustPlatform = targetPkgs.makeRustPlatform {
    cargo = rustChannel;
    rustc = rustChannel;
  };
in
  rustPlatform.buildRustPackage rec {
    pname = "nestalgic";
    version = "0.0.1";

    src = builtins.filterSource
      (path: type: type != "directory" || builtins.baseNameOf path != "target")
      ./.;

    inherit target;

    cargoSha256 = "0jacm96l1gw9nxwavqi1x4669cg6lzy9hr18zjpwlcyb3qkw9z7f";
  }
