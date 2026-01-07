{
  description = "High Performance YouTube Downloader";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            yt-dlp
            ffmpeg
            cargo-watch
            cargo-nextest
          ];

          shellHook = ''
            echo "🚀 High Performance YouTube Downloader Dev Environment"
            echo "   Rust: $(rustc --version)"
            echo "   yt-dlp: $(yt-dlp --version)"
            echo "   ffmpeg: $(ffmpeg -version | head -1)"
          '';

          RUST_BACKTRACE = 1;
          RUST_LOG = "info,tower_http=debug,axum=debug,hpyd_server=debug,hpyd_downloads=debug";
        };
      });
}

