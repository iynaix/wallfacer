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
  libiconv,
  libsoup_3,
  openssl,
  pango,
  tailwindcss,
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

  cudaSupport ? false,
  rocmSupport ? false,
}:
assert !(cudaSupport && rocmSupport);
rustPlatform.buildRustPackage {
  pname = "wallfacer";
  inherit version;

  src = ../../.;

  cargoLock = {
    lockFile = ../../Cargo.lock;
    outputHashes = {
      "dioxus-sdk-0.5.0" = "sha256-ox/vWTfyrPYnfvHjEX+nc+OdKGA4Aa2yQsfMzFJ6e8s=";
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
