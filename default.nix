{ crossSystem ? null, target ? null, targetPkgsOverride ? null }:

let
  sources = import nix/sources.nix;
  hostPkgs = import sources.nixpkgs { overlays = [(import sources.rust-overlay)]; };

  # Target packages _don't_ import the mozilla-nixpkgs overlay as it overrides `buildRustPackage` with
  # a variant that doesn't respect the `target` attribute.
  #
  # We need the `target` attribute for windows builds so we can override it with `x86_64-pc-windows-gnu`.
  # Normally we could just take the target triple but `mingwW64`'s triple doesn't match with a supported
  # rust toolchain.
  targetPkgs = if targetPkgsOverride == null then hostPkgs else targetPkgsOverride;

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


    # A list of dependencies whose host platform is the new derivation's build platform
    #
    # a.k.a build time dependencies
    nativeBuildInputs = [
      hostPkgs.pkg-config
      targetPkgs.xorg.libX11
    ];

    # a.k.a run time dependencies
    buildInputs = [
      targetPkgs.xorg.libX11
    ];

    # Patch for "failed to create directory `/homeless-shelter/.cargo/...`" issue.
    # See: https://github.com/NixOS/nixpkgs/issues/61618
    preConfigure = ''
      export HOME=`mktemp -d`
    '';

    src = builtins.filterSource
      (path: type: type != "directory" || builtins.baseNameOf path != "target")
      ./.;

    inherit target;

    doCheck = false;

    cargoSha256 = "1l012vfg5ncrgn7kmq9knl74vv0ljvzlhb4lwin1h86wl8z2a0dz";
  }
