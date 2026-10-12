## Context

See proposal.md for the why. The facts in the code that shape the approach:

- **The app runs an immediate-mode loop** (eframe/egui) on one thread. The document lives in `AppState` and its `Workspace`, and every document change goes through `Workspace::record(label, before, now, coalesce)`, which stores a before/after `Snapshot` in `History`. Undo labels are `&'static str`.
- **Textures are `Surface`s** in `Project::surfaces`. Each surface knows its vehicle texture through `SurfaceTemplate { package_id, texture_id, part }`, and its size in texture pixels. A project can also be editing a symbol (`Project::editing_symbol`), in which case the canvas shows the symbol, not a texture.
- **Import already exists as a function of bytes:** `import::read_bytes(file_name, bytes)` reads PNG, JPEG and SVG, and assets are deduplicated by content.
- **tp-render renders a surface to a `Pixmap` off the UI thread** (Export Texture, thumbnails), and `encode_png` turns it into PNG bytes.
- **The workspace has no async runtime and no HTTP server.** Dependencies are kept lean, and `unsafe_code` is forbidden.
- **Gestures:** `gesture.rs` knows whether a drag or transform is active and whether it edits the document. Text editing sessions are in `text_session.rs`.

## Goals / Non-Goals

**Goals:**
- The LLM never sees a half-applied document, and the player never loses a gesture to a concurrent call.
- Every write call is one undo step, reverted with ⌘Z as a whole.
- The protocol layer is testable without egui: tool calls in, document changes out.
- The UI thread never blocks on the network or on rendering.

**Non-Goals:**
- Streaming progress, MCP resources or prompts, sampling, notifications to the client. Only `tools/list` and `tools/call` (plus `initialize` and `ping`) are implemented.
- Several clients editing at once in a coordinated way. Calls are serialized; two clients simply take turns.
- Changing the persisted project format. Nothing new is saved.

## Decisions

### 1. Transport: Streamable HTTP on 127.0.0.1, with a minimal blocking server

TruckPaint is already open with a project when the LLM wants to use it, so the stdio transport (where the client spawns the server process) doesn't fit. Streamable HTTP on `http://127.0.0.1:<port>/mcp` lets any client connect to the running app.

The server answers each POST with a single JSON response (`Content-Type: application/json`), which the Streamable HTTP transport allows. No SSE stream is opened, since the server sends no notifications. GET on `/mcp` returns 405.

**Implementation:** a small blocking HTTP server (`tiny_http`) on its own thread, with JSON-RPC handled by hand with serde_json.
- *Alternative: the `rmcp` SDK.* It is the official Rust SDK, but it brings tokio and axum for a handful of methods. It is worth revisiting if resources, notifications or a built-in chat client are added.
- *Alternative: stdio plus a shim binary* (`truckpaint mcp` that forwards to the running app over a local socket). It works with clients that only speak stdio, but it adds a second binary and an IPC. Most clients now speak HTTP. It can be added later on top of the same server.

**Port:** 7373 by default (a free value; the exact number can change). If the port is taken, the server reports it in Preferences ("Port 7373 is in use") and stays off. The port is editable in Preferences. A random port would break the client's saved config at each launch.

**Security:**
- bind to `127.0.0.1` only, never `0.0.0.0`;
- reject any request whose `Origin` header is present and is not `http://127.0.0.1:<port>` or `http://localhost:<port>`, to block web pages (DNS rebinding);
- off by default; the player turns it on.

No token is required: the server is local, off by default, and any local process could already read the project files. A token can be added later without changing the tools.

### 2. Threading: the server thread never touches the document

```
 MCP client                 server thread (tp-mcp)                 UI thread (tp-app)
 ----------                 -----------------------                -------------------
 POST tools/call  ------->  parse JSON-RPC, validate args
                            build a Request (typed, tp-core types)
                            send (Request, reply_tx) ---channel--> each frame: if no gesture,
                            request_repaint()                       drain requests one by one,
                            wait on reply_rx (timeout 30 s)          apply on the Project,
                                                                     record one history step,
                            <--------------------------------------- send Reply
 <----- JSON response       encode result / error
```

- The **server thread** parses and validates arguments into typed requests (`tp-core` types: `Frame`, `Paint`, `PathData`…), so the UI thread does only cheap, already-checked work. Parsing SVG path data happens here.
- The **UI thread** drains the channel at the start of each frame, **unless a gesture is in progress** (a drag, a transform, a pen path being drawn, a text editing session, a live panel edit, or a modal dialog other than Preferences, About, Keyboard Shortcuts and messages: Preferences is where the player turns the server on, and blocking calls there would make the first call time out). Requests then wait in the channel and run on the first frame after the gesture ends. This is what "calls wait for the player's gesture" means.
- **Read requests** get a cheap clone of what they need (`Arc`-based objects and surfaces) and the reply is built on the server thread.
- **Render requests** take a clone of the surface and its assets on the UI thread, and the render and PNG encoding run on the server thread, so the UI never waits for tiny-skia.
- **Timeout:** if the UI thread doesn't answer within 30 s (a long gesture, a modal dialog left open), the call fails with "TruckPaint is busy (a dialog or an edit is in progress); try again". The request is marked cancelled so it isn't applied late.
- `egui::Context` is `Send + Sync`, and the server thread holds a clone to call `request_repaint()` so an idle app wakes up.

### 3. One call, one undo step, with a fixed set of labels

Each write tool maps to one `Workspace::record` call with a static label from a fixed table ("AI: Add Shape", "AI: Add Path", "AI: Add Text", "AI: Import Image", "AI: Update Object", "AI: Arrange", "AI: Delete", "AI: Add Swatch", "AI: Apply Style", "AI: Place Symbol"). These labels are localized like the others. Coalescing is off: two calls are two steps, even if they come within a second.

The selection is set to the objects the call created or changed, as a paste would. This makes it obvious on the canvas what the LLM just did, and Undo restores the previous selection as usual.

A failed call changes nothing and records nothing: the request is applied on a clone of the project and swapped in only when every step succeeded.

### 4. Addressing: textures by id, objects by id, everything in texture pixels

- `get_project` returns each texture as `{ id, vehicle, name, part, width, height }`. The `id` is `"<package_id>/<texture_id>"`, which is stable across sessions and readable by the LLM. Surfaces without a template (unit tests only) get their index.
- Every write tool takes `texture` explicitly. It never depends on the texture the player has open, so the player can keep working on another texture while the LLM draws.
- Objects are addressed by their `ObjectId` as a number. Objects inside groups are addressable.
- Coordinates are texture pixels, origin at the top-left, y down, as in the rulers. A shape is given by `x, y, width, height` (its unrotated box) plus an optional `rotation` in degrees. This maps to `Frame`.
- Colors are hex strings (`#rrggbb` or `#rrggbbaa`), or `{ "swatch": "<name or id>" }` to link to a palette swatch.

While a symbol is being edited, the project's textures are still addressable, but write calls are refused (`"A symbol is being edited; finish editing it first"`). This avoids changing instances while their symbol is open.

### 5. Paths from SVG path data

`add_path` takes `d` (SVG path syntax: M, L, H, V, C, S, Q, T, A, Z, absolute and relative) in texture pixels. kurbo's `BezPath::from_svg` parses it, arcs are converted to cubics, and the result becomes a `Path` object whose frame is fitted to its bounds (`Object::refit`). Several subpaths in one `d` make one compound path, which keeps holes (letter counters, rings) with the even-odd or non-zero rule given by `fill_rule`.

*Alternative:* accepting a whole SVG document and converting every element. That means mapping SVG styling, transforms, groups and clips to the object model, a large surface for little gain: the LLM can call `add_path` once per shape, and a whole SVG can still be imported as an image asset with `import_image`.

### 6. Images: base64 or path, read once, through the existing import

`import_image` takes either `data` (base64) and a `name`, or `path` (an absolute local path). Exactly one must be given.
- A path is read once on the server thread, with a size cap (50 MB) and only if its extension is `.png`, `.jpg`, `.jpeg` or `.svg`. Nothing else on disk is read, and nothing is written.
- The bytes go through `import::read_bytes`, so formats, error messages and asset deduplication are the ones of File › Import.
- Placement: at `x, y, width, height` when given. Otherwise the image keeps its aspect ratio and is centered on the texture at the size File › Import would place it.

### 7. Renders sent back to the LLM

`render_texture` and `get_template` return an MCP `image` content (base64 PNG). Images are expensive in the LLM's context, so the output is capped: `max_size` defaults to 1024 px on the longest side, and is capped at 2048. The tool's text content states the scale ("rendered at 1024 × 512, 1 px = 4 texture px") so the LLM can convert what it sees back into texture pixels.

`render_texture` takes `with_template` (default true): the artwork on the artboard color, with the template over it at its opacity, as the canvas shows it (the canvas draws the template over the artwork). The template matters for placing things on the truck.

### 8. Tool descriptions carry the livery rules

The tool descriptions (what the LLM reads in `tools/list`) explain the domain in a few lines:
- each texture is a UV layout of one vehicle, and positions don't carry over between vehicles;
- look at the template before drawing;
- reuse the palette and symbols for elements shared across the fleet;
- every call is undoable by the player.

The server's `initialize` result also carries a short `instructions` text with this workflow (read project, look at the template, draw, render, compare, correct). This is where the quality of the "recreate this livery" case is won, at no cost in code.

### 9. Preferences and status

- Preferences stores `mcp_enabled: bool` (default false) and `mcp_port: u16`. Both are read with defaults, so older preference files still load.
- Turning the switch on starts the server thread. Turning it off stops it: the listener is closed, pending requests get an error, and the thread is joined.
- The status bar item is computed from a small shared state (`Arc<Mutex<McpStatus>>`): off, listening, or connected (a request in the last 60 s), plus the number of AI changes in the last minute.

## Risks / Trade-offs

- [The LLM draws poorly from a photo: bad proportions, wrong placement on the UV layout] → The see tools close the loop (render, compare, correct). The tool descriptions and `instructions` push that workflow. Every call is undoable. The result depends on the model, and the change promises tools, not livery quality.
- [Large renders flood the LLM's context] → Default 1024 px, cap 2048, and the scale is stated so smaller renders stay usable.
- [A web page calls the local server] → Bound to 127.0.0.1, Origin check, off by default.
- [The UI thread is busy (modal dialog, long gesture) and calls pile up] → 30 s timeout with a clear message, and cancelled requests are dropped, not applied late.
- [Path data from the LLM is huge or degenerate (NaN, thousands of points)] → Arguments are validated on the server thread: finite numbers, a cap on path segments per call (10 000) and on objects per call, and a clear error instead of a panic.
- [Many small calls make a long undo list] → One step per call is the contract, and History is bounded (200 steps). A batch tool can come later if needed.
- [Port conflicts] → Fixed default port, editable, and the conflict is shown in Preferences instead of silently picking another port.
- [`tiny_http` limits] → It is blocking, with one thread per connection pool; for a single local client this is enough. Moving to `rmcp` later changes only `tp-mcp`'s transport, not the tools or the app side.

## Migration Plan

Nothing to migrate: the server is off by default, no project data changes, and preferences gain two optional fields read with defaults. Rollback is removing the crate and the Preferences section.
