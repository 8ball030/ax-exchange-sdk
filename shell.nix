let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  packages = with pkgs; [
    # rustup alone, so rust-toolchain.toml picks the compiler
    rustup
  ];
}
