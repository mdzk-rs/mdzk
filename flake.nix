{
  description = "Plain text Zettelkasten based on mdBook";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self
    , nixpkgs
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;

      nixpkgsFor = forAllSystems (system: import nixpkgs {
        inherit system;
        overlays = [ self.overlays.default ];
      });

      pname = "mdzk";
      version =
        (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
    in
    {
      overlays.default = final: prev: {
        mdzk = prev.callPackage ./nix/package.nix { inherit prev pname version; };
      };

      apps = forAllSystems (system:
        let
          pkgs = nixpkgsFor."${system}";
          app = {
            type = "app";
            program = "${pkgs.mdzk}/bin/mdzk";
          };
        in
        {
          # `nix run`
          default = app;

          # `nix run .#mdzk`
          mdzk = app;
        });

      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor."${system}";
        in
        {
          # `nix build`
          default = pkgs.mdzk;

          # `nix build .#mdzk`
          "${pname}" = pkgs.mdzk;

          # `nix build .#website`
          website = pkgs.callPackage ./nix/website.nix { };
        });

      formatter = forAllSystems (system:
        let
          pkgs = nixpkgsFor."${system}";
        in
        # `nix fmt`
        pkgs.nixpkgs-fmt);

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgsFor."${system}";

          inherit (pkgs)
            mkShell
            cargo
            rustc
            rust-analyzer;
        in
        {
          default = mkShell {
            buildInputs = [ cargo rust-analyzer rustc ];
          };
        });
    };
}
