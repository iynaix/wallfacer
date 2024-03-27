{
  version,
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook,
  atk,
  cairo,
  gdk-pixbuf,
  glib,
  gtk3,
  libsoup_3,
  openssl,
  pango,
  tailwindcss,
  webkitgtk_4_1,
  xdotool,
  stdenv,
  darwin,
  realcugan-ncnn-vulkan,
  anime-face-detector,
  oxipng,
  jpegoptim,
  installShellFiles,
  makeWrapper,
}:
rustPlatform.buildRustPackage {
  pname = "wallpaper-ui";
  inherit version;

  src = ../../.;

  cargoLock = {
    lockFile = ../../Cargo.lock;
    outputHashes = {
      "dioxus-free-icons-0.8.0" = "sha256-YjBPeRWLq0n1M3LFplOFv4F2Z90hKjzdSCikB9g958M=";
    };
  };

  env.NIX_RELEASE_VERSION = version;

  postPatch = ''
    substituteInPlace src/main.rs \
      --replace "public/tailwind.css" "$out/public/tailwind.css"
  '';

  preBuild = ''
    mkdir -p $out/public

    tailwindcss \
      --minify \
      --input input.css \
      --output $out/public/tailwind.css
  '';

  nativeBuildInputs = [
    tailwindcss
    pkg-config
    wrapGAppsHook
    xdotool
    installShellFiles
    makeWrapper
  ];

  buildInputs =
    [
      atk
      cairo
      gdk-pixbuf
      glib
      gtk3
      libsoup_3
      openssl
      pango
      webkitgtk_4_1
      xdotool
    ]
    ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.AppKit
      darwin.apple_sdk.frameworks.CoreGraphics
      darwin.apple_sdk.frameworks.Foundation
      darwin.apple_sdk.frameworks.Security
    ];

  preFixup = ''
    installShellCompletion \
      --bash completions/*.bash \
      --fish completions/*.fish \
      --zsh completions/_*
  '';

  postFixup = ''
    # add own path so wallpaper-pipeline can run wallpaper-ui
    wrapProgram $out/bin/wallpaper-pipeline \
      --prefix PATH : "$out/bin" \
      --prefix PATH : "${
        lib.makeBinPath [
          realcugan-ncnn-vulkan
          anime-face-detector
          oxipng
          jpegoptim
        ]
      }"

    # FIXME: GDK_BACKEND=x11 is required for keyboard shortcuts to work?
    wrapProgram $out/bin/wallpaper-ui \
      --set WEBKIT_DISABLE_COMPOSITING_MODE 1 \
      --set GDK_BACKEND x11
  '';

  meta = with lib; {
    description = "";
    homepage = "https://github.com/iynaix/wallpaper-ui";
    license = licenses.mit;
    maintainers = with maintainers; [ iynaix ];
    mainProgram = "wallpaper-ui";
  };
}
