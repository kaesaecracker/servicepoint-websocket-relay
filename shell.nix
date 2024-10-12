{pkgs ? import <nixpkgs> {}}: let
  rust-toolchain = pkgs.symlinkJoin {
    name = "rust-toolchain";
    paths = with pkgs; [rustc cargo rustPlatform.rustcSrc rustfmt clippy];
  };
in
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [
      rust-toolchain
    ];

    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  }
