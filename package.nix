{
  version,
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook3,
  dioxus-cli,
  atk,
  cairo,
  gdk-pixbuf,
  glib,
  gtk3,
  libiconv,
  libsoup_3,
  openssl,
  pango,
  webkitgtk_4_1,
  xdotool,
  gexiv2,
  stdenv,
  darwin,
  realcugan-ncnn-vulkan,
  anime-face-detector,
  oxipng,
  jpegoptim,
  libwebp,
  installShellFiles,
  makeWrapper,
  # for tailwindcss
  nodejs,
  buildNpmPackage,

  cudaSupport ? false,
  rocmSupport ? false,
}:
let
  tailwind_css = buildNpmPackage {
    pname = "tailwind_css";
    version = "0.1.0";

    # NOTE: the rust files are needed for tailwind to extract css from
    src = ./.;

    npmDepsHash = "sha256-AyuTztrp8M7bmwbxrlWWn5Ga4UfLzkKeI4BkyhFQVfY=";

    nativeBuildInputs = [ nodejs ];

    buildPhase = ''
      runHook preBuild

      npx tailwindcss --minify --input ./input.css -o ./tailwind.css

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      mkdir -p $out

      cp -r tailwind.css $out/

      runHook postInstall
    '';
  };
in
assert !(cudaSupport && rocmSupport);
rustPlatform.buildRustPackage {
  pname = "wallfacer";
  inherit version;

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "dioxus-sdk-0.7.0" = "sha256-7wIYZDpMdMbMDKXgn+++JzwvU0yrIUWjwSsZCN4zhVA=";
    };
  };

  env.NIX_RELEASE_VERSION = version;

  # build the css, then build the app with dx so bundling still works
  buildPhase = ''
    mkdir -p public

    cp ${tailwind_css}/tailwind.css public/tailwind.css

    dx build --platform desktop --release
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp -r target/dx/wallfacer/release/linux/app/* $out/bin
  '';

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook3
    xdotool
    installShellFiles
    makeWrapper
    dioxus-cli
  ];

  buildInputs = [
    atk
    cairo
    gdk-pixbuf
    glib
    gtk3
    libiconv
    libsoup_3
    openssl
    pango
    webkitgtk_4_1
    xdotool
    gexiv2 # for reading metadata
  ]
  ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.AppKit
    darwin.apple_sdk.frameworks.CoreGraphics
    darwin.apple_sdk.frameworks.Foundation
    darwin.apple_sdk.frameworks.Security
  ];

  postFixup = ''
    installShellCompletion --cmd wallfacer \
      --bash <($out/bin/wallfacer --generate bash) \
      --fish <($out/bin/wallfacer --generate fish) \
      --zsh <($out/bin/wallfacer --generate zsh)

    installManPage target/man/*

    # FIXME: GDK_BACKEND=x11 is required for keyboard shortcuts to work?
    wrapProgram $out/bin/wallfacer \
      --set WEBKIT_DISABLE_COMPOSITING_MODE 1 \
      --prefix PATH : "${
        lib.makeBinPath [
          realcugan-ncnn-vulkan
          (anime-face-detector.override { inherit cudaSupport rocmSupport; })
          libwebp
          oxipng
          jpegoptim
        ]
      }"
  '';

  meta = with lib; {
    description = "";
    homepage = "https://github.com/iynaix/wallfacer";
    license = licenses.mit;
    maintainers = with maintainers; [ iynaix ];
    mainProgram = "wallfacer";
  };
}
