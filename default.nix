{ pkgs, pname, version, rust-toolchain, ... }:

pkgs.rustPlatform.buildRustPackage {
  inherit pname version;

  src = pkgs.lib.cleanSource ./.;

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.CoreServices
  ];
  nativeBuildInputs = [ rust-toolchain ];

  cargoSha256 = "sha256-wLCcri7QyxL5WpgIx9s8DoGo2BhC5CbKFzTinLAIAec=";
}
