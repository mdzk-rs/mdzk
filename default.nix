{ pkgs, pname, version, rust-toolchain, ... }:

pkgs.rustPlatform.buildRustPackage {
  inherit pname version;

  src = pkgs.lib.cleanSource ./.;

  nativeBuildInputs = [ rust-toolchain ];

  cargoSha256 = "sha256-kfqsqFuDNgYMAQM2AD8tliW0norvBrcRQtrwzJRw8tY=";
}
