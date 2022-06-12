{ pkgs, pname, version, ... }:

let
  rustPlatform = pkgs.makeRustPlatform {
    rustc = pkgs.customRustToolchain;
    cargo = pkgs.customRustToolchain;
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

  cargoSha256 = "sha256-O4FrETbZ/2NkcTi7OARwXw/NqVgR4WuxrgIbR4jP5lk=";
}
