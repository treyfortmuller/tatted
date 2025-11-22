{
  description = "A simple flake-based rust project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      system = "x86_64-linux";
      overlays = [ rust-overlay.overlays.default ];
      pkgs = import nixpkgs { inherit system overlays; };

      rust = pkgs.rust-bin.stable.latest.default.override {
        # Optional extensions can be added here
        extensions = [ ]; # e.g. "llvm-tools-preview"
        targets = [ ]; # e.g. "thumbv7em-none-eabihf"
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rust
        ];

        # Optional: helpful environment variables for Rust dev
        # RUST_BACKTRACE = "1";
        RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;

        shellHook = ''
          echo "🦀 Evolved into a crab again... shit."
          rustc --version
        '';
      };

      formatter.${system} = pkgs.nixfmt-tree;
    };
}
