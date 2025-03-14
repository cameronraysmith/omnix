# A dummy flake to cache test dependencies in Nix store.
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # NOTE: These inputs should kept in sync with those used in the Rust source (cli.rs)
    haskell-multi-nix.url = "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321";
    services-flake.url = "github:juspay/services-flake/23cf162387af041035072ee4a9de20f8408907cb";
    nixos-config.url = "github:srid/nixos-config/fe9c16cc6a60bbc17646c15c8ce3c5380239ab92";

    # FIXME: Sadly, these will still result in rate-limiting errors, due to the 60/hour limit.
    # See https://github.com/NixOS/nix/issues/5409

    # system_list.rs tests
    nix-systems-empty.url = "github:nix-systems/empty";
    # Used in `om init` tests
    haskell-flake.url = "github:srid/haskell-flake";
    haskell-template.url = "github:srid/haskell-template";
    rust-nix-template.url = "github:srid/rust-nix-template";
    nixos-unified-template.url = "github:juspay/nixos-unified-template";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      perSystem = { system, ... }: {
        packages = {
          haskell-multi-nix = inputs.haskell-multi-nix.packages.${system}.default;
        };
      };
    };
}
