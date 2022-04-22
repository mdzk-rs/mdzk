{ pkgs, pname, version, ... }:

let
  rustPlatform = pkgs.makeRustPlatform {
    rustc = pkgs.rustc;
    cargo = pkgs.cargo;
  };
in rustPlatform.buildRustPackage {
  inherit pname version;

  src = pkgs.lib.cleanSource ../.;

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin
    [ pkgs.darwin.apple_sdk.frameworks.CoreServices ];

  makeFlags = [ "PREFIX=$(out)" ];

  postBuild = ''
    pandoc --standalone --to man public/man.md -o public/mdzk.1
  '';

  preInstall = ''
    install -d $out/share/man/man1/
    install -pm 0644 public/mdzk.1 $out/share/man/man1/
  '';

  nativeBuildInputs = with pkgs; [ pandoc ];

  cargoSha256 = "sha256-HPEu1yv2ebOQVxmQBtLM6SpjiVdbgeJWBYkXcPUUXpY=";
}
