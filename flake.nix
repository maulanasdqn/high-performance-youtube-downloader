{
  description = "High Performance YouTube Downloader";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, rust-overlay, ... }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # Pre-fetch swagger UI zip to avoid network access during build
          swaggerUiZip = pkgs.fetchurl {
            url = "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.14.zip";
            sha256 = "sha256-SBJE0IEgl7Efuu73n3HZQrFxYX+cn5UU5jrL4T5xzNw=";
          };

          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type: (craneLib.filterCargoSources path type);
          };

          commonArgs = {
            pname = "hpyd";
            version = "0.1.0";
            inherit src;
            strictDeps = true;

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            OPENSSL_NO_VENDOR = 1;

            # Copy swagger UI zip to build dir before build
            preBuild = ''
              mkdir -p $TMPDIR/swagger-ui
              cp ${swaggerUiZip} $TMPDIR/swagger-ui/v5.17.14.zip
              chmod 644 $TMPDIR/swagger-ui/v5.17.14.zip
              export SWAGGER_UI_DOWNLOAD_URL="file://$TMPDIR/swagger-ui/v5.17.14.zip"
            '';
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          hpyd-server = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--package hpyd-server";
          });
        in
        {
          default = hpyd-server;
          inherit hpyd-server;
        }
      );

      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.hpyd;
        in
        {
          options.services.hpyd = {
            enable = lib.mkEnableOption "HPYD Server";
            port = lib.mkOption {
              type = lib.types.int;
              default = 3200;
            };
            host = lib.mkOption {
              type = lib.types.str;
              default = "127.0.0.1";
            };
            environmentFile = lib.mkOption {
              type = lib.types.nullOr lib.types.path;
              default = null;
            };
            nginx = {
              enable = lib.mkEnableOption "nginx reverse proxy";
              domain = lib.mkOption {
                type = lib.types.str;
                default = "hpyd.localhost";
              };
              enableSSL = lib.mkOption {
                type = lib.types.bool;
                default = true;
              };
              acmeEmail = lib.mkOption {
                type = lib.types.str;
                default = "";
              };
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.hpyd = {
              description = "High Performance YouTube Downloader";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];

              serviceConfig = {
                Type = "simple";
                ExecStart = "${self.packages.${pkgs.system}.hpyd-server}/bin/hpyd-server";
                Restart = "always";
                RestartSec = 5;
                DynamicUser = true;
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
                StateDirectory = "hpyd";
                WorkingDirectory = "/var/lib/hpyd";
              } // lib.optionalAttrs (cfg.environmentFile != null) {
                EnvironmentFile = cfg.environmentFile;
              };

              environment = {
                HOST = cfg.host;
                PORT = toString cfg.port;
                RUST_LOG = "info";
              };

              path = with pkgs; [ yt-dlp ffmpeg ];
            };

            services.nginx = lib.mkIf cfg.nginx.enable {
              enable = true;
              virtualHosts.${cfg.nginx.domain} = {
                enableACME = cfg.nginx.enableSSL;
                forceSSL = cfg.nginx.enableSSL;
                locations."/" = {
                  proxyPass = "http://${cfg.host}:${toString cfg.port}";
                  proxyWebsockets = true;
                  extraConfig = ''
                    proxy_set_header X-Real-IP $remote_addr;
                    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                    proxy_set_header X-Forwarded-Proto $scheme;
                  '';
                };
              };
            };

            security.acme.certs = lib.mkIf (cfg.nginx.enable && cfg.nginx.enableSSL) {
              ${cfg.nginx.domain}.email = cfg.nginx.acmeEmail;
            };
          };
        };

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };
        in
        {
          default = pkgs.mkShell {
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
            '';

            RUST_BACKTRACE = 1;
            RUST_LOG = "info,tower_http=debug,axum=debug,hpyd_server=debug";
          };
        }
      );
    };
}
