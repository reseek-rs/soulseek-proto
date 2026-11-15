{
  description = "soulseek-proto, the Soulseek wire protocol as Rust types";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      inherit (nixpkgs) lib;
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = lib.genAttrs systems;
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          # cargo-fuzz builds with -Zsanitizer, which only nightly accepts
          nightly = pkgs.rust-bin.selectLatestNightlyWith (
            toolchain: toolchain.minimal.override { extensions = [ "rust-src" ]; }
          );
          # the oldest Rust the manifest's rust-version promises, which rust-overlay names in full
          rustVersion = (lib.importTOML ./Cargo.toml).package.rust-version;
          msrv =
            pkgs.rust-bin.stable.${
              if lib.length (lib.splitString "." rustVersion) == 2 then "${rustVersion}.0" else rustVersion
            }.minimal;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              cargo-nextest
              just
              git
            ];

            env.RUST_BACKTRACE = "1";
          };

          msrv = pkgs.mkShell { packages = [ msrv ]; };

          fuzz = pkgs.mkShell {
            packages = [
              nightly
              pkgs.cargo-fuzz
              pkgs.just
            ];
          };
        }
      );

      formatter = forAllSystems (system: (pkgsFor system).nixfmt);
    };
}
