# Linux

Psychopomp renders through wgpu, which uses Vulkan on Linux. Nothing native is
linked at build time: wgpu, winit, and xkbcommon load the Vulkan loader, Wayland,
X11, and xkbcommon at runtime, and `ring` needs a C compiler.

## With Nix

The flake's dev shell provides Rust, FFmpeg with `libx264`, Bun, ImageMagick,
`uv`, `espeak-ng`, and `vulkaninfo`, and puts the runtime libraries on
`LD_LIBRARY_PATH`. It works on NixOS and on other distributions.

```sh
nix develop --command cargo build --release
nix develop --command vulkaninfo --summary        # check that your GPU appears
nix develop --command cargo run -p psychopomp-hello
nix develop --command cargo run --release -- plan render target/hello.json output/hello.mp4 --theme neutral
nix develop --command cargo run --release -- plan present target/hello.json
```

The host's Vulkan drivers and layers are built against the host's libc and may
not load into a Nix-built process, so the shell selects Nixpkgs' Mesa drivers
(RADV, ANV, NVK, lavapipe) and Mesa's layers through `VK_DRIVER_FILES` and
`VK_IMPLICIT_LAYER_PATH`. If `VK_DRIVER_FILES` or `VK_ICD_FILENAMES` is already
set, the shell leaves both alone. For NVIDIA's proprietary driver, provide its
ICD yourself, for example through nixGL.

If the renderer reports `GPU: llvmpipe`, Vulkan found no hardware device. It
still renders, on the CPU, much more slowly. Check that `/dev/dri`
exists and that you can access it. Containers and sandboxes often hide it.

## Without Nix

Install a Rust toolchain (1.87 or newer), a C compiler, FFmpeg with `libx264`,
[Bun](https://bun.sh), ImageMagick 7, `uv`, and your distribution's Vulkan
loader and Mesa drivers. On Arch, apart from Bun:

```sh
pacman -S --needed rustup base-devel ffmpeg imagemagick uv espeak-ng \
  vulkan-icd-loader vulkan-radeon vulkan-tools wayland libxkbcommon
```

## Narration

`scripts/narrate.ts` picks local tools by platform:

| | macOS | Linux and others |
| --- | --- | --- |
| `--draft` voice | `say` (`SAY_VOICE`, default `Samantha`) | `espeak-ng` (`ESPEAK_VOICE`, default `en-us`) |
| Word timings | `mlx-whisper` | faster-whisper through `whisper-ctranslate2` (model `large-v3`) |

`WHISPER_BACKEND=mlx|faster` and `WHISPER_MODEL` override the transcriber. An
installed `whisper-ctranslate2` is used directly; otherwise it runs through
`uvx`. The flake's `narrate` shell includes it, so it needs no Python setup:

```sh
nix develop .#narrate --command bun scripts/narrate.ts scenes/<scene>/narration/script.json --draft
```

The first run downloads the model, about 3 GB, into `~/.cache/huggingface`. It
runs on the CPU; set `WHISPER_MODEL=small` for faster, rougher drafts.

Clips timed by anything other than mlx-whisper's default model record it in
`narration.json` as `"whisper"`, and `psychopomp-media adopt` refuses them, because
a Media Lock keys Whisper timings by that model. The Fish engine runs the
`fish-say` helper at `FISH_SAY`, which defaults to
`~/.opencode/skill/fish-audio/scripts/fish-say.ts`.

`psychopomp-media` still drafts with `say` and times with `mlx-whisper`, so
`PSYCHOPOMP_MEDIA=draft` and Whisper-timed voices (Fish, `say`, or ElevenLabs
`.whisper()`) only generate on macOS. Lines that ElevenLabs aligns itself, and
resources already in a lock, work everywhere.

## Fonts

CommitMono is bundled, so code and labels match on every machine. Prose and
display faces (`Face::Sans`, `Serif`, `Light`, `Shout`, and headers that use
them) ask for Helvetica Neue and Didot, which macOS installs. Elsewhere
cosmic-text substitutes an installed face, so those frames differ from macOS
renders. Contact sheets label tiles with `SHEET_FONT`, else Menlo on macOS, else
the bundled CommitMono.

## Determinism across platforms

Rendering is deterministic on one machine. Across platforms, `libm` can differ
in a float's last digits. Scene Programs that commit their reel, such as
`balls-v3`, can rewrite it with last-digit changes when run on Linux. Compare
`verify` baselines only against baselines from the same machine.
