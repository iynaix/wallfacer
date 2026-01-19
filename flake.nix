{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    anime-face-detector.url = "github:iynaix/yolov8-animeface-cli";
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      self,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          # config,
          # self',
          # inputs',
          pkgs,
          system,
          ...
        }:
        let
          # custom packages here
          anime-face-detector = inputs.anime-face-detector.packages.${system}.default;
        in
        {
          # Per-system attributes can be defined here. The self' and inputs'
          # module parameters provide easy access to attributes of the same
          # system.
          devShells.default = pkgs.mkShell {
            packages =
              with pkgs;
              # wallfacer specific dependencies
              [
                cargo-edit
                oxipng
                jpegoptim
                libwebp
                realcugan-ncnn-vulkan
                anime-face-detector
                dioxus-cli
                nodejs
                # helper shell scripts
                (writeShellScriptBin "tailwind" "tailwindcss -i ./input.css -o ./public/tailwind.css --watch")
                (writeShellScriptBin "dev" "dx serve --platform desktop --hot-reload")
                (writeShellScriptBin "rsx" ''dx translate --raw "$@"'')
              ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            }
            // {
              XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}";
              GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules/";
            };

            # for tailwind binary
            shellHook = ''
              export PATH="$PWD/node_modules/.bin:$PATH"
            '';

            nativeBuildInputs = with pkgs; [
              cargo
              rustc
              rust-analyzer
              rustfmt
              clippy
              pkg-config
            ];

            buildInputs =
              with pkgs;
              # dioxus dependencies
              [
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
                gexiv2 # for reading metadata
              ];
          };

          packages = rec {
            default = pkgs.callPackage ./package.nix {
              inherit anime-face-detector;
              version =
                if self ? "shortRev" then
                  self.shortRev
                else
                  nixpkgs.lib.replaceStrings [ "-dirty" ] [ "" ] self.dirtyShortRev;
            };
            wallfacer = default;
            wallfacer-cuda = default.override { cudaSupport = true; };
            wallfacer-rocm = default.override { rocmSupport = true; };
          };
        };
    };
}
