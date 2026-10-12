## Why

Players often start from an image: a photo of a real company truck, a mockup, or a logo the company already has. Turning it into a livery on a vehicle's texture layout is slow and manual. LLMs with vision can read such an image and can draw well (they write SVG paths fluently), but today they have no way to act on a TruckPaint project.

Exposing TruckPaint as a **local MCP server** lets the player use the LLM client they already have (Claude Desktop, Claude Code or any other MCP client) to draw on the open project: place a supplied logo, or recreate a livery from a reference image. TruckPaint embeds no LLM, stores no API key and has no chat panel. This change covers the tools layer, and a built-in chat could use the same tools later.

It serves the livery editor, not general drawing. The tools speak in vehicles, textures and texture pixels. The LLM sees each texture's template, so it can place elements where they land on the truck.

## What Changes

- **A local MCP server** inside the running app, over Streamable HTTP on `127.0.0.1` only. It is **off by default**. Preferences gets an **AI assistant (MCP)** section with an on/off switch, the server's URL with a Copy button, and a short line on how to add it to an MCP client. The setting persists.
- **Read tools** (no effect on the document):
  - the open project: game, vehicles, textures (with their ids and sizes), palette, styles, symbols;
  - the object tree of a texture, and one object's properties.
- **See tools:**
  - a texture's template as a PNG;
  - a texture's artwork rendered as a PNG, with or without the template underneath, with a bounded size.
  
  Together they let the LLM compare its result with the reference image and correct it.
- **Write tools**, each acting on one texture named explicitly, in texture pixels:
  - add a rectangle, ellipse, polygon or star;
  - add a path from SVG path data (`d`), converted to an editable path;
  - add a text;
  - import an image (PNG, JPEG or SVG), either as base64 data or as a local file path read once, and place it;
  - update an object (frame, fill as a color, gradient or swatch, stroke, opacity, shadow, name, visibility);
  - reorder, group and ungroup, align, combine (boolean operations), and delete objects;
  - add a palette swatch, apply a style, place a symbol instance.
- **Undoable, one step per call.** Every write call is one undo step labeled "AI: …" (for example "AI: Add Path"). The player can undo anything the LLM did with ⌘Z / Ctrl+Z.
- **No writes to disk.** The LLM can't save, export a texture or a mod, create, open or close a project, or change preferences. Reading an image file by path for import is the only file access.
- **Safe coexistence with the player:**
  - calls that arrive during a gesture (a drag, a text editing session) wait until it ends;
  - with no project open, or while a symbol is edited, write calls fail with a clear message;
  - the status bar shows the server's state ("MCP: on", "MCP: connected") and how many AI changes the last minute brought, with an icon, never by color alone.

Non-goals:
- an LLM inside the app, a chat panel or API keys;
- remote access (any address other than `127.0.0.1`), authentication or several projects at once;
- tools that save, export, open or close files;
- automatic vectorization of bitmaps (the LLM draws paths itself);
- driving the UI (tool selection, zoom, spaces).

## Capabilities

### New Capabilities
- `mcp-server`: the local MCP server and its tools:
  - how it is enabled, where it listens, and what clients see;
  - the read, see and write tools, their inputs and errors, and their addressing by texture and texture pixels;
  - one undo step per write call, and no file writes;
  - waiting for the player's gestures, and behavior with no project or while a symbol is edited.

### Modified Capabilities
- `app-preferences`: the MCP server setting is persisted, and the Preferences dialog gets the AI assistant (MCP) section.
- `workspace-layout`: the status bar shows the MCP server's state when it is on.
- `undo-history`: changes made through the MCP server are undoable like any other, as one step per call, with an "AI: …" label.
- `image-import`: images can also be imported through the MCP server, from base64 data or a file path, with the same formats, errors and deduplication.

## Impact

- **New crate `tp-mcp`:**
  - the MCP protocol (JSON-RPC over Streamable HTTP), the tool schemas, and the conversion between tool arguments and `tp-core` types;
  - parsing SVG path data (`d`) into `PathData` with kurbo, already a dependency.
- **tp-app:**
  - starts and stops the server thread from Preferences;
  - applies tool calls on the UI thread through a channel, as one history step each, and wakes egui with `request_repaint`;
  - the Preferences section, the status bar item, and strings in en/fr/es/de.
- **tp-render:** renders a texture (and its template) to PNG for the see tools. This path exists for Export Texture and thumbnails.
- **Dependencies:** a small HTTP server and JSON (serde_json is already one). The choice between a minimal blocking server and the `rmcp` SDK (which brings tokio) is in the design.
- **Security:** listening on `127.0.0.1` only and off by default. The Origin header is checked to block web pages from calling the server (DNS rebinding).
- **docs/roadmap.md:** adds `mcp-server` to the next changes.
