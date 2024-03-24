{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    anime-face-detector.url = "github:iynaix/anime-face-detector";
  };

  outputs =
    {
      nixpkgs,
      devenv,
      systems,
      self,
      ...
    }@inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devenv-up = self.devShells.x86_64-linux.default.config.procfileScript;

      devShells = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          libraries = with pkgs; [
            atk
            cairo
            dbus
            gdk-pixbuf
            glib
            gtk3
            libappindicator
            libsoup_3
            openssl_3
            pango
            webkitgtk_4_1
            xdotool
          ];
        in
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;

            modules = [
              {
                packages =
                  with pkgs;
                  # pipeline dependencies
                  [
                    oxipng
                    jpegoptim
                    inputs.anime-face-detector.packages.${system}.anime-face-detector
                    (callPackage ./nix/realcugan-ncnn-vulkan { })
                  ]
                  ++ [
                    pkg-config
                    tailwindcss
                    (dioxus-cli.overrideAttrs (o: rec {
                      version = "0.5.0-alpha.2";

                      src = pkgs.fetchCrate {
                        inherit (o) pname;
                        inherit version;
                        hash = "sha256-ACvWXDx844f0kSKVhrZ0VLImjRfcGu45BIFtXP5Tf5I=";
                      };

                      checkFlags = [ "--skip=cli::autoformat::test_auto_fmt" ];

                      cargoDeps = pkgs.rustPlatform.importCargoLock {
                        lockFile = src + "/Cargo.lock";
                        # allowBuiltinFetchGit = true;
                      };
                    }))
                  ]
                  ++ libraries;

                env = {
                  XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS";
                  # FIXME: fix lag on wayland?
                  # https://github.com/tauri-apps/tauri/issues/7354#issuecomment-1620910100
                  WEBKIT_DISABLE_COMPOSITING_MODE = 1;
                  # GDK_BACKEND = "x11";
                };

                languages = {
                  javascript = {
                    enable = true;
                    npm.enable = true;
                  };
                  rust.enable = true;
                };

                processes = {
                  # workaround so the tailwind task doesn't exit immediately
                  tailwind.exec = "(while true; do sleep 10; done) | tailwindcss -i ./input.css -o ./public/tailwind.css --watch";
                  # dev.exec = "dx serve --platform desktop";
                };

                scripts = {
                  tailwind.exec = "tailwindcss -i ./input.css -o ./public/tailwind.css --watch";
                  dev.exec = "dx serve --platform desktop --hot-reload";
                  rsx.exec = ''dx translate --raw "$@"'';
                };
              }
            ];
          };
        }
      );
    };
}
