let
  sources = import ./nix/sources.nix;
  rustOverlay = import sources.rust-overlay;
  pkgs = import <nixpkgs> {
    overlays = [ rustOverlay ];
  };
in
pkgs.mkShell {
  buildInputs = [
    pkgs.rust-bin.stable.latest.default
    pkgs.rust-analyzer
    pkgs.clippy
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
  ];

  RUST_BACKTRACE = 1;

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
