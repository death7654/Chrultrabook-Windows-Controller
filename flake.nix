{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (pkgs.rust-bin.fromRustupToolchain {
              channel = "stable"; # feel free to change the channel
              components = [
                "rustfmt"
                "rust-src"
              ];
              targets = [ "wasm32-unknown-unknown" ];
              profile = "minimal";
            })

            cargo-deny
            cargo-edit
            cargo-watch
            openssl_3
            pkg-config
            rust-analyzer

            # Tauri specific
            cairo
            curl
            dbus
            gdk-pixbuf
            glib
            gtk3
            libappindicator
            libayatana-appindicator
            librsvg
            libsoup_2_4
            nodejs_22
            systemdLibs
            webkitgtk_4_1
            wget
          ];
        };

        packages.default = pkgs.callPackage ./default.nix {
          lib = nixpkgs.lib;
          inherit pkgs;
        };
      }
    );

}
