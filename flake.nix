{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    devenv.url = "github:cachix/devenv";
    anime-face-detector.url = "github:iynaix/anime-face-detector";
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      self,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devenv.flakeModule ];
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let
          # custom packages here
          inherit (inputs.anime-face-detector.packages.${system}) anime-face-detector;
          realcugan-ncnn-vulkan = (pkgs.callPackage ./nix/realcugan-ncnn-vulkan { });
          catppuccin-tailwindcss =
            (pkgs.callPackage ./nix/catppuccin-tailwindcss { })."@catppuccin/tailwindcss";
          # custom tailwind with prebaked catppuccin
          tailwindcss-with-catppuccin = pkgs.nodePackages.tailwindcss.overrideAttrs (o: {
            plugins = [ catppuccin-tailwindcss ];
          });
        in
        {
          # Per-system attributes can be defined here. The self' and inputs'
          # module parameters provide easy access to attributes of the same
          # system.
          devenv.shells.default = {
            packages =
              with pkgs;
              # pipeline dependencies
              [
                oxipng
                jpegoptim
                realcugan-ncnn-vulkan
                anime-face-detector
              ]
              ++ [
                pkg-config
                tailwindcss-with-catppuccin
                dioxus-cli
              ]
              ++ [
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

            env = {
              XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS";
              # FIXME: fix lag on wayland?
              # https://github.com/tauri-apps/tauri/issues/7354#issuecomment-1620910100
              WEBKIT_DISABLE_COMPOSITING_MODE = 1;
              # FIXME: keyboard shortcuts do not seem to work without this option
              GDK_BACKEND = "x11";
            };

            languages.rust.enable = true;

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
          };

          packages = rec {
            default = pkgs.callPackage ./nix/wallpaper-ui {
              inherit realcugan-ncnn-vulkan anime-face-detector;
              tailwindcss = tailwindcss-with-catppuccin;
              version =
                if self ? "shortRev" then
                  self.shortRev
                else
                  nixpkgs.lib.replaceStrings [ "-dirty" ] [ "" ] self.dirtyShortRev;
            };
            wallpaper-ui = default;
          };
        };
    };
}
