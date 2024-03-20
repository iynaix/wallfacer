{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv/rust-rewrite";
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
                  [
                    dioxus-cli
                    pkg-config
                    tailwindcss
                  ]
                  ++ libraries;

                env = {
                  XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS";
                  # FIXME: fix lag on wayland?
                  # https://github.com/tauri-apps/tauri/issues/7354#issuecomment-1620910100
                  WEBKIT_DISABLE_COMPOSITING_MODE = 1;
                  GDK_BACKEND = "x11";
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
                  dev.exec = "dx serve --platform desktop";
                };

                scripts = {
                  tailwind.exec = "tailwindcss -i ./input.css -o ./public/tailwind.css --watch";
                  dev.exec = "dx serve --platform desktop";
                };
              }
            ];
          };
        }
      );
    };
}
