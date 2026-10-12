# TruckPaint

A livery editor for **Euro Truck Simulator 2** and **American Truck Simulator**: pick a
vehicle, design its paint job with vector tools, and export a ready-to-install mod.

> Early development. The current build designs the paint jobs of a fleet: vehicle
> packages with their templates, projects holding several trucks and trailers of one
> game, vector editing (shapes, paths, text, images, gradients), texture export to
> PNG or DDS, and export of the whole fleet as a ready-to-install mod (`.scs`).

Built in Rust with [egui](https://github.com/emilk/egui) (wgpu backend) for Linux,
Windows and macOS.

## Drawing with an AI assistant (MCP)

TruckPaint can let an AI assistant draw on the open project through a local
[MCP](https://modelcontextprotocol.io) server: recreate a livery from a reference
image, or place a logo you give it. The assistant runs in your own MCP client;
TruckPaint holds no API key.

1. In **Preferences › AI assistant (MCP)**, check **Let an AI assistant draw on the
   open project**. The server listens on `http://127.0.0.1:7373/mcp` (loopback only;
   the port can be changed while it is off). It is off by default.
2. Add it to your client:
   - **Claude Code**: `claude mcp add --transport http truckpaint http://127.0.0.1:7373/mcp`
   - **Claude Desktop**, or any client that only speaks stdio, through
     [mcp-remote](https://www.npmjs.com/package/mcp-remote), in
     `claude_desktop_config.json`:
     ```json
     { "mcpServers": { "truckpaint": { "command": "npx", "args": ["mcp-remote", "http://127.0.0.1:7373/mcp"] } } }
     ```
   - Any other client: the Streamable HTTP transport at that address.
3. Open a project and ask, for example: *"Paint the Standard cab like this photo"*.

The assistant reads the project, looks at each texture's template, draws shapes, SVG
paths, texts and images in texture pixels, and renders the texture to check its work.
Every change it makes is one undo step labeled "AI: …". It can't save, export or open
files; the status bar shows when it is connected and how many changes it made in the
last minute.

## Building

Runtimes and commands are managed by [mise](https://mise.jdx.dev): `mise.toml` pins
Rust (with rustfmt and clippy) and defines the project tasks.

```sh
mise install       # install the pinned toolchain
mise run run       # run the app (debug build)
mise run build     # release build
mise tasks         # list all tasks
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
mise run fmt       # format
mise run lint      # clippy, warnings denied
mise run test      # unit + headless UI tests
mise run ci        # everything CI runs, in order
```

CI runs the same mise tasks on Linux, Windows and macOS.

Changes that only touch documentation (`*.md` files, `openspec/`, `.claude/`) skip
format, lint, tests and builds; any other file runs everything. Check a range locally
with `mise run ci:changes <base> <head>` (self-test: `mise run ci:changes:test`).
Branch protection should require the **CI result** check, which passes when the Rust
jobs pass or are skipped for a documentation-only change.

UI behavior is tested headlessly with `egui_kittest`. To render screenshots of the main
screens for visual review (needs a GPU), run:

```sh
mise run screenshots   # images in target/screenshots/
```

Logs are written to stderr and to a daily file in the application data directory
(`logs/`). Set `RUST_LOG=debug` for more detail.

### Workspace layout

| Crate      | Role                                                                 |
|------------|----------------------------------------------------------------------|
| `tp-core`  | Domain types. No UI, filesystem or threads.                          |
| `tp-ui`    | Design system: tokens, theme, embedded fonts/icons, widgets.         |
| `tp-app`   | The desktop application: screens, commands, layout, preferences.     |
| `tp-vehicles` | Vehicle packages (`.tpv`): manifest, validation, templates.       |
| `tp-pack`  | The `tpv` command: builds and checks vehicle packages.               |
| `tp-mcp`   | The local MCP server: protocol, tools and their arguments (no UI).   |

Planned changes are tracked with OpenSpec in `openspec/`; product decisions and the
order of upcoming changes are in [docs/roadmap.md](docs/roadmap.md).

### Vehicle packages

Vehicles come as packages (`.tpv`), described in
[docs/vehicle-package-format.md](docs/vehicle-package-format.md). The repository
ships a sample, **TruckPaint Sample Truck**, in [examples/vehicles/](examples/vehicles/)
(versions 1.0.0 and 1.1.0, to try Update Template); the app also offers to install it
while no vehicle is installed.

```sh
mise run pack <folder> [-o file.tpv]   # pack a folder (PNG, SVG or DDS templates)
mise run sample-vehicles               # rebuild the sample packages
```

## Licenses

Code: MIT. Bundled fonts: [Geist](https://github.com/vercel/geist-font),
[JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) and the document fonts such as
[Inter](https://rsms.me/inter/) (SIL Open Font License, see `assets/fonts/*-LICENSE.txt`),
and [Phosphor Icons](https://phosphoricons.com/) (MIT).
