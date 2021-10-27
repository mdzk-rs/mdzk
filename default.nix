{ pkgs, pname, version, rust-toolchain, ... }:

pkgs.rustPlatform.buildRustPackage {
  inherit pname version;

  src = pkgs.lib.cleanSource ./.;

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.CoreServices ]; 
  nativeBuildInputs = [ rust-toolchain ];

  cargoSha256 = "sha256-kfqsqFuDNgYMAQM2AD8tliW0norvBrcRQtrwzJRw8tY=";
}
