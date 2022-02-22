{ pkgs, pname, version, rust-toolchain, ... }:

pkgs.rustPlatform.buildRustPackage {
  inherit pname version;

  src = pkgs.lib.cleanSource ./.;

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [ 
    pkgs.darwin.apple_sdk.frameworks.CoreServices 
  ]; 
  nativeBuildInputs = [ rust-toolchain ];

  cargoSha256 = "sha256-CiA8Z1+S6+Lwms70IiRvIN83gValHuy6kHOukR2O7/Q=";
}
