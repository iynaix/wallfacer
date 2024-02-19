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
      ...
    }@inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
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
                # env = {
                #   LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH";
                #   XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS";
                # };

                packages =
                  with pkgs;
                  [
                    dioxus-cli
                    pkg-config
                  ]
                  ++ libraries;

                # https://devenv.sh/reference/options/
                # dotenv.disableHint = true;

                languages.rust.enable = true;
              }
            ];
          };
        }
      );
    };
}
