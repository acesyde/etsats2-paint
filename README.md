# TruckPaint

A livery editor for **Euro Truck Simulator 2** and **American Truck Simulator**: pick a
vehicle, design its paint job with vector tools, and export a ready-to-install mod.

> Early development. The current build contains the application shell: home screen,
> New Project dialog, editor workspace layout, design system, keyboard shortcuts and
> preferences. Canvas editing, vehicle templates and export come next.

Built in Rust with [egui](https://github.com/emilk/egui) (wgpu backend) for Linux,
Windows and macOS.

## Building

The toolchain is pinned in `rust-toolchain.toml` (also managed by `mise`).

```sh
cargo run -p tp-app            # debug build
cargo run -p tp-app --release  # optimized build
```

The executable is named `truckpaint`.

### Platform prerequisites

**Linux** (Debian/Ubuntu package names):

```sh
sudo apt-get install libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev \
  libxrandr-dev libxi-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libgl1-mesa-dev
```

Both X11 and Wayland sessions are supported.

**Windows**: the MSVC toolchain (Visual Studio Build Tools with the C++ workload).

**macOS**: Xcode Command Line Tools (`xcode-select --install`).

### Graphics troubleshooting

TruckPaint renders with Vulkan, DirectX 12 or Metal. If it fails to start with a
graphics adapter error (old drivers, virtual machines), force another backend:

```sh
WGPU_BACKEND=gl truckpaint      # OpenGL (Linux, Windows)
WGPU_BACKEND=vulkan truckpaint  # Vulkan
```

## Development

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

UI behavior is tested headlessly with `egui_kittest`. To render screenshots of the main
screens for visual review (needs a GPU), run:

```sh
cargo test -p tp-app --test screenshots -- --ignored
# images in target/screenshots/
```

Logs are written to stderr and to a daily file in the application data directory
(`logs/`). Set `RUST_LOG=debug` for more detail.

### Workspace layout

| Crate      | Role                                                                 |
|------------|----------------------------------------------------------------------|
| `tp-core`  | Domain types. No UI, filesystem or threads.                          |
| `tp-ui`    | Design system: tokens, theme, embedded fonts/icons, widgets.         |
| `tp-app`   | The desktop application: screens, commands, layout, preferences.     |

Planned changes are tracked with OpenSpec in `openspec/`.

## Licenses

Code: MIT. Bundled fonts: [Inter](https://rsms.me/inter/) (SIL Open Font License,
see `assets/fonts/Inter-LICENSE.txt`) and [Phosphor Icons](https://phosphoricons.com/) (MIT).
