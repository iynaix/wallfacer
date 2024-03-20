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

                # https://devenv.sh/reference/options/
                # dotenv.disableHint = true;

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
                  dev.exec = "dx serve --platform desktop";
                };
              }
            ];
          };
        }
      );
    };
}
