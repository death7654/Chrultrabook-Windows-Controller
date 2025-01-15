{
  lib,
  pkgs,
  ...
}:
with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "chrultrabook-tools";
  version = (builtins.fromJSON (builtins.readFile (src + "/package-lock.json"))).version;
  src = ./.;

  #fontSrc = fetchurl {
  #  url = "https://fonts.googleapis.com/css2?family=Open+Sans:wdth,wght@75..100,300..800&display=swap";
  #  hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  #};

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./src-tauri/Cargo.lock;
  };

  npmDeps = importNpmLock {
    npmRoot = ./.;
  };

  nativeBuildInputs = [
    cargo-tauri.hook
    nodejs
    importNpmLock.npmConfigHook
    pkg-config
    wrapGAppsHook4
  ];

  buildInputs =
    [ openssl_3 ]
    ++ lib.optionals stdenv.hostPlatform.isLinux [
      glib-networking # Most Tauri apps need networking
      webkitgtk_4_1
    ];

  cargoRoot = "src-tauri";
  buildAndTestSubdir = cargoRoot;

}
