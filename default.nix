{ pkgs, pname, version, rust-toolchain, ... }:

pkgs.rustPlatform.buildRustPackage {
  inherit pname version;

  src = pkgs.lib.cleanSource ./.;

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [ 
    pkgs.darwin.apple_sdk.frameworks.CoreServices 
  ]; 
  nativeBuildInputs = [ rust-toolchain ];

  cargoSha256 = "sha256-uJ00tGiKtcYghFUh0fcYg4nZc/o8yhvlVs+6/aRNY5s=";
}
