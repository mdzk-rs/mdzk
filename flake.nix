{
  description = "Plain text Zettelkasten based on mdBook";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem
      (system:
        let
          name = "mdzk";

          pkgs = import nixpkgs {
            inherit system;
          };

          naersk-lib = naersk.lib."${system}";

          mdzk-pkg = naersk-lib.buildPackage {
            pname = name;
            root = ./.;
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
            nativeBuildInputs = with pkgs; [ rustc cargo rust-analyzer ];
          };
        });
}
