{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
        name = "mdzk";
      in
      rec {
        # `nix build`
        packages.${name} = naersk-lib.buildPackage {
          pname = name;
          root = ./.;
        };
        defaultPackage = packages.${name};

        # `nix run`
        apps.${name} = utils.lib.mkApp {
          drv = packages.${name};
        };
        defaultApp = apps.${name};

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
        };
      });
}
