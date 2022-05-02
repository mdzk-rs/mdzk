{ pkgs, ... }:

with pkgs;
stdenv.mkDerivation {
  name = "website";

  src = ../public;

  buildInputs = [ gnumake pandoc ];

  installPhase = ''
    mkdir -p $out
    make
    cp -r website/. $out
  '';
}
