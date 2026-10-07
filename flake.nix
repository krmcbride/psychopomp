{
  description = "Psychopomp: code-first motion graphics in Rust";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (
        pkgs:
        let
          inherit (pkgs) lib stdenv;
          # wgpu, winit, and xkbcommon dlopen these at runtime; nothing links
          # them at build time, so they only need to be on the loader path.
          runtimeLibs = with pkgs; [
            vulkan-loader
            libGL
            wayland
            libxkbcommon
            libx11
            libxcursor
            libxi
            libxrandr
            libxcb
          ];
        in
        rec {
          default = pkgs.mkShell {
            packages =
              (with pkgs; [
                cargo
                rustc
                clippy
                rustfmt
                rust-analyzer
                pkg-config
                ffmpeg # with libx264
                bun
                imagemagick
                uv # uvx runs Whisper for narration word timings
              ])
              ++ lib.optionals stdenv.hostPlatform.isLinux (
                with pkgs;
                [
                  espeak-ng # draft narration voice where macOS `say` is absent
                  vulkan-tools # vulkaninfo, for checking the adapter
                ]
              );

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

            shellHook = lib.optionalString stdenv.hostPlatform.isLinux ''
              export LD_LIBRARY_PATH="${lib.makeLibraryPath runtimeLibs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
              # The host's Vulkan drivers and layers are built against the host's
              # libc and may not load into a Nix-built process, so use Nixpkgs'
              # Mesa drivers (RADV, ANV, NVK, lavapipe) and its layers unless the
              # user chose others, for example NVIDIA's ICD through nixGL.
              if [ -z "''${VK_DRIVER_FILES:-}" ] && [ -z "''${VK_ICD_FILENAMES:-}" ]; then
                export VK_DRIVER_FILES="$(echo ${pkgs.mesa}/share/vulkan/icd.d/*.json | tr ' ' :)"
                export VK_IMPLICIT_LAYER_PATH="${pkgs.mesa}/share/vulkan/implicit_layer.d"
              fi
              if [ -z "''${__EGL_VENDOR_LIBRARY_FILENAMES:-}" ]; then
                export __EGL_VENDOR_LIBRARY_FILENAMES="${pkgs.mesa}/share/glvnd/egl_vendor.d/50_mesa.json"
              fi
            '';
          };

          # Narration: faster-whisper word timings (large, so not in the default shell).
          narrate = pkgs.mkShell {
            inputsFrom = [ default ];
            packages = lib.optionals stdenv.hostPlatform.isLinux [ pkgs.whisper-ctranslate2 ];
            inherit (default) RUST_SRC_PATH;
          };
        }
      );

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
