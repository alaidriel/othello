{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem
    (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit overlays;
        inherit system;
      };
      rust-stable = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src"];
      };
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs.darwin.apple_sdk; [
          rust-stable
          pkgs.postgresql
          frameworks.SystemConfiguration
        ];

        DATABASE_URL = "postgres://othello-server:password@0.0.0.0:5432/othello-server";

        # Yes, these are plaintext database credentials. It makes local development easier.
        shellHook = ''
          export PATH="$PATH:$HOME/.local/share/cargo/bin"
        '';
      };
    });
}
