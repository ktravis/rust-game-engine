{
  description = "development shell with runtime dependencies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # rust-overlay.url = "github:oxalica/rust-overlay";
    # crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    # crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        # craneLib = crane.mkLib pkgs;
        deps = with pkgs; [
          libGL
          wayland
          libxkbcommon
          vulkan-headers
          vulkan-loader
          vulkan-validation-layers
          vulkan-tools-lunarg

          openssl
          pkg-config
          gdb
          renderdoc
        ];
        # };
        # commonArgs = {
        #   src = craneLib.cleanCargoSource ./.;
        #   strictDeps = true;
        #
        #   buildInputs = buildPackages;
        # cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        # crate = craneLib.buildPackage (
        #   commonArgs
        #   // {
        #     src = nixpkgs.lib.fileset.toSource {
        #       root = ./.;
        #       fileset = nixpkgs.lib.fileset.unions [
        #         (craneLib.fileset.commonCargoSources ./.)
        #         (nixpkgs.lib.fileset.maybeMissing ./testdata)
        #         (nixpkgs.lib.fileset.maybeMissing ./res)
        #       ];
        #     };
        #   }
        # );
      in {
        # checks = {
        #   inherit crate;
        #
        #   crate-clippy = craneLib.cargoClippy (
        #     commonArgs
        #     // {
        #       inherit cargoArtifacts;
        #       cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        #     }
        #   );
        #   crate-fmt = craneLib.cargoFmt {
        #     src = craneLib.cleanCargoSource ./.;
        #   };
        # };
        # packages.default = crate;
        # apps.default = flake-utils.lib.mkApp {
        #   drv = crate;
        #   exePath = "/bin/test-game";
        # };
        devShells.default = pkgs.mkShell {
          packages = deps;
          RUST_BACKTRACE = "1";
          RUST_LOG = "warn";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath deps}";
          # VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          # VULKAN_SDK = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          # XDG_DATA_DIRS = builtins.getEnv "XDG_DATA_DIRS";
          # XDG_RUNTIME_DIR = "/run/user/1000";
        };
      }
    );
}
