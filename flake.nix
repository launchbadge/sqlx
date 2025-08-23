{
  description = "Rust with WebAssembly";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
        in
          {
            devShells.default =
              pkgs.mkShell rec {
                name = "wasm-devshell";
                nativeBuildInputs = [ pkgs.pkg-config ];
                buildInputs = with pkgs; [
                  clang
                  llvmPackages.bintools
                  nodejs_21
                  wasm-pack
                  trunk
                  (rust-bin.stable.latest.default.override {
                    targets = [ "wasm32-unknown-unknown" ];
                  })
                  sqlx-cli
                  openssl
                  trunk
                ];

                # Add precompiled library to rustc search path
                RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
                  pkgs.sqlite
                ]);

                LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);
                
                # Add glibc, clang, glib, and other headers to bindgen search path
                BINDGEN_EXTRA_CLANG_ARGS =
                # Includes normal include path
                (builtins.map (a: ''-I"${a}/include"'') [
                  pkgs.glibc.dev
                  pkgs.sqlite.dev
                ])
                # Includes with special directory paths
                ++ [
                  ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
                  ''-I"${pkgs.glib.dev}/include/glib-2.0"''
                  ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
                ];

                CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
                LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
              };
          }
      );
}
