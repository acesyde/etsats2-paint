## ADDED Requirements

### Requirement: AI assistant (MCP) setting
The Preferences dialog SHALL have an **AI assistant (MCP)** section with:
- a switch that turns the local MCP server on or off (see the mcp-server capability), applied at once, off by default;
- the port (1024 to 65535, default 7373), editable while the switch is off;
- the server's URL (`http://127.0.0.1:<port>/mcp`) in a read-only field with a **Copy** button;
- a line saying that an MCP client (such as Claude Desktop or Claude Code) can connect to this URL to draw on the open project, and that every change it makes can be undone;
- the server's state: off, on, or "Port <port> is in use".

The switch and the port SHALL be persisted with the other preferences. A preferences file without them SHALL load with the switch off and the default port. "Reset to defaults" SHALL NOT change them.

#### Scenario: Turning the server on
- **WHEN** the user turns on AI assistant (MCP) in Preferences
- **THEN** the server starts, the section shows it is on, and Copy puts `http://127.0.0.1:7373/mcp` on the clipboard

#### Scenario: Setting survives restart
- **WHEN** the user turns the server on, quits and restarts the application
- **THEN** the server is on again with the same port

#### Scenario: Older preferences file
- **WHEN** the application starts with a preferences file written before this setting existed
- **THEN** all its other preferences load, the server is off, and no message about the preferences appears
