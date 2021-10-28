{
  description = "Plain text Zettelkasten based on mdBook";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    rust.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust }:
    utils.lib.eachDefaultSystem (system:
      let
        pname = "mdzk";
        version =
          (builtins.fromTOML
            (builtins.readFile ./Cargo.toml)).package.version;

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust) ];
        };

        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-std" "rust-src" ];
        };

        mdzk-pkg = pkgs.callPackage ./default.nix {
          inherit pkgs pname version rust-toolchain;
        };
      in
      rec {
        # `nix build`
        packages.${pname} = mdzk-pkg;
        defaultPackage = packages.${pname};

        # `nix run`
        apps.${pname} = utils.lib.mkApp {
          drv = packages.${pname};
        };
        defaultApp = apps.${pname};

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rust-toolchain
            rust-analyzer
          ];
        };
      }
    );
}
