{
  description = "Plain text Zettelkasten based on mdBook";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    rust.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    rust,
  }: let
    pname = "mdzk";
    version =
      (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
  in
    {
      overlays.default = nixpkgs.lib.composeManyExtensions [
        rust.overlays.default
        (final: _: {
          customRustToolchain =
            final.rust-bin.selectLatestNightlyWith
            (toolchain:
              toolchain.default.override {
                extensions = ["rust-std" "rust-src"];
              });

          mdzk = import ./nix/package.nix {
            inherit pname version;
            pkgs = final;
          };
        })
      ];
    }
    // utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [self.overlays.default];
      };

      inherit (pkgs) mdzk;
    in rec {
      # `nix build .#mdzk`
      packages.${pname} = mdzk;

      # `nix build .#website`
      packages.website = pkgs.callPackage ./nix/website.nix {inherit pkgs;};

      # `nix build`
      packages.default = packages.${pname};

      # `nix run`
      apps.${pname} = utils.lib.mkApp {drv = packages.${pname};};
      apps.default = apps.${pname};

      # `nix develop`
      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          # rust
          customRustToolchain
        ];
      };
    });
}
