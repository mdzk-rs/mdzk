{
  description = "Plain text Zettelkasten based on mdBook";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    rust.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, naersk, rust }:
    utils.lib.eachDefaultSystem
      (
        system:
          let
            name = "mdzk";

            rust-overlay = import rust;

            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay ];
            };

            rust-pkg = pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-std"
                "rust-src"
              ];
            };

            naersk-lib = naersk.lib."${system}".override {
              rustc = rust-pkg;
            };

            mdzk-pkg = naersk-lib.buildPackage {
              pname = name;
              root = pkgs.lib.cleanSource ./.;
            };
          in
            rec {
              # `nix build`
              packages.${name} = mdzk-pkg;
              defaultPackage = packages.${name};

              # `nix run`
              apps.${name} = utils.lib.mkApp {
                drv = packages.${name};
              };
              defaultApp = apps.${name};

              # `nix develop`
              devShell = pkgs.mkShell {
                nativeBuildInputs = with pkgs; [
                  rust-pkg
                  rust-analyzer
                ];
              };
            }
      );
}
