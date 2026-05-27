{
  description = "Gildos — an AI-native operating system (Phase-1 scaffold)";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets    = [ "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          name = "gildos-dev";
          buildInputs = with pkgs; [
            rust
            cargo-nextest
            cargo-deny
            pkg-config
            clang_18
            lld_18
            qemu_full
            wasmtime
            jq
            sqlite
            lmdb
          ];
          shellHook = ''
            echo "gildos dev shell ready"
            echo "  cargo check        — typecheck the workspace"
            echo "  cargo nextest run  — run unit tests"
            echo "  see CONTRIBUTING.md for the four-tier ladder (§17)"
          '';
        };
      });
}
