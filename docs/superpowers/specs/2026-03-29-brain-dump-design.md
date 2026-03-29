# brain-dump — Design Spec

## Context

Steven generates ideas faster than he can drill them down into concrete action plans. Existing tools (notes, docs, task managers) don't connect high-level vision to subtasks and cross-project dependencies in a way that feels natural. brain-dump is a personal desktop app for capturing ideas, breaking them into structured plans, and visualizing how everything connects — with Claude Code as the AI interface for brainstorming and planning.

## Architecture

**Tauri 2 desktop app** — Rust backend, SvelteKit webview, SQLite storage.

### Crate Structure

```
brain-dump/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── brain-dump-core/          # Shared library: DB, models, queries
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db.rs             # SQLite connection, migrations, WAL setup
│   │   │   ├── models.rs         # Node, Edge, Tag, StatusHistory structs
│   │   │   ├── queries.rs        # All CRUD operations
│   │   │   └── templates.rs      # Template loading/rendering
│   │   └── Cargo.toml
│   ├── brain-dump-cli/           # CLI binary for Claude Code
│   │   ├── src/main.rs           # clap-based CLI
│   │   └── Cargo.toml
│   └── brain-dump-app/           # Tauri 2 desktop app
│       ├── src/
│       │   ├── main.rs
│       │   └── commands.rs       # Tauri invoke handlers
│       ├── Cargo.toml
│       └── tauri.conf.json
├── ui/                           # SvelteKit frontend
│   ├── src/
│   │   ├── routes/
│   │   │   ├── +layout.svelte    # Sidebar + main area
│   │   │   ├── +page.svelte      # Dashboard (project card grid)
│   │   │   └── project/[id]/
│   │   │       └── +page.svelte  # Project detail (split + tabs)
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   │   ├── Sidebar.svelte
│   │   │   │   ├── ProjectCard.svelte
│   │   │   │   ├── PhaseCard.svelte
│   │   │   │   ├── TaskItem.svelte
│   │   │   │   ├── MarkdownRenderer.svelte
│   │   │   │   ├── ConnectionBadge.svelte
│   │   │   │   └── TagBadge.svelte
│   │   │   ├── stores/
│   │   │   │   └── projects.svelte.ts   # Svelte 5 rune-based state
│   │   │   └── tauri.ts                 # Tauri invoke() wrappers
│   │   └── app.css                      # OpsML phosphor-green theme
│   ├── svelte.config.js                 # Static adapter for Tauri
│   ├── vite.config.ts
│   └── package.json
└── templates/
    ├── project.md
    ├── phase.md
    └── task.md
```

### Key Technical Decisions

- **Tauri 2** — latest stable, Rust-native, small binary (~10MB)
- **rusqlite** with WAL mode — concurrent reads from CLI + app without locking
- **ULID** for IDs — sortable by creation time, URL-friendly, no collisions
- **No HTTP server** — Tauri IPC for UI, direct SQLite for CLI. Both go through `brain-dump-core`
- **SvelteKit static adapter** — builds to static HTML/JS for Tauri webview, no SSR
- **Svelte 5 runes** exclusively ($state, $derived, $props, $effect)
- **Tailwind CSS v4** — CSS-based config (`@theme` in `app.css`), no `tailwind.config.ts`
- **Templates** — embedded at compile time via `include_str!()`, no runtime file lookup
- **Migrations** — `rusqlite` `user_version` pragma. Each schema version gets a migration function. On startup, `brain-dump-core` reads `PRAGMA user_version`, applies any pending migrations sequentially, and bumps the version.
- **DB location**: `~/.brain-dump/brain-dump.db` (global, not per-repo)

## Data Model

### `nodes`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (ULID) | Primary key |
| type | TEXT | `project` \| `phase` \| `task` |
| title | TEXT | Display name |
| description | TEXT | Markdown content |
| status | TEXT | `active` \| `completed` \| `archived` |
| sort_order | INTEGER | Manual ordering within a parent (default 0) |
| created_at | TEXT | ISO8601 |
| updated_at | TEXT | ISO8601 |

### `edges`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (ULID) | Primary key |
| source_id | TEXT | FK → nodes.id (CASCADE delete) |
| target_id | TEXT | FK → nodes.id (CASCADE delete) |
| edge_type | TEXT | `parent` \| `blocks` \| `related` |
| created_at | TEXT | ISO8601 |

**Constraints**: UNIQUE(source_id, target_id, edge_type). Index on (source_id, edge_type) and (target_id, edge_type).

### `tags`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (ULID) | Primary key |
| name | TEXT | UNIQUE |
| color | TEXT | Hex or OKLCH for display |

### `node_tags`

| Column | Type | Notes |
|--------|------|-------|
| node_id | TEXT | FK → nodes.id (CASCADE delete) |
| tag_id | TEXT | FK → tags.id (CASCADE delete) |

**Constraint**: PRIMARY KEY(node_id, tag_id)

### `status_history`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (ULID) | Primary key |
| node_id | TEXT | FK → nodes.id (CASCADE delete) |
| old_status | TEXT | Previous status |
| new_status | TEXT | New status |
| changed_at | TEXT | ISO8601 |

Automatically populated by `brain-dump-core` whenever a node's status changes. No v1 UI consumer — this is passive logging for future timeline/analytics views.

### Hierarchy Rules

- 3 levels max: Project → Phase → Task
- `parent` edges: source = parent, target = child. Example: Project "OpsML v3" → Phase "Core Server" is stored as `(source=opsml_v3_id, target=core_server_id, edge_type=parent)`. This reads naturally as "parent contains child."
- `blocks` edges: source blocks target (source must complete before target can start). Cycle detection enforced at write time — reject any edge that would create a cycle. Cross-type blocking allowed (task blocks phase in another project).
- `related` edges: bidirectional soft link (stored once, queried both directions)

### Computed States

- **Phase progress**: count(completed child tasks) / count(total child tasks)
- **Project progress**: derived from phase progress
- **Blocked**: a node is blocked if it has an incoming `blocks` edge from an uncompleted node
- No separate columns — computed at query time or in the UI

## UI Design

### Theme

Port OpsML's phosphor-green CRT dark theme:
- Background: near-black charcoal (oklch 5.5%-15%, 150deg hue)
- Primary: phosphor green (#8ddb9f / oklch 82% 0.14 152)
- Borders: `border-2` with semi-transparent green
- Shadows: hard-offset neo-brutalist (3px 3px 0 0 color)
- Font: JetBrains Mono
- Effects: scanline overlay (2px repeating, 6% opacity), vignette, phosphor bloom text-shadow
- All colors via CSS custom properties in OKLCH

Reference: `/Users/stevenforrester/Documents/GitHub/opsml/crates/opsml_server/opsml_ui/opsml-theme.css`

### Dashboard (Home)

- **Sidebar**: Collapsible icon sidebar (Home, Projects, Tags, Settings). Hover to expand, pin/unpin toggle. Matches OpsML's sidebar pattern.
- **Main area**: Responsive card grid. 1 column on small screens, 2 on medium, max 3 on large. Cards do NOT stretch full-width on XL screens.
- **Project card**: Title, description snippet (truncated), phase/task count, progress bar, tags as badges. Neo-brutalist card style with hard shadow. Hover: translate + shadow removal.
- **Top area**: "brain-dump" title + "New Project" button + filter/search bar.
- **Empty state**: Single dashed-border card prompting to create first project.

### Project Detail

- **Header**: Back button, project title, status badge, Edit/Archive actions.
- **Split panel layout**:
  - **Left (40%)**: Rendered markdown overview. Scrollable. Always visible.
  - **Right (60%)**: Tabbed interface:
    - **Phases & Tasks tab**: Phases as expandable accordion cards. Each shows title, progress bar (segmented), task count. Expanded: task list with checkmarks, status, dependency badges (e.g., "blocks: UI Migration" in red).
    - **Connections tab**: List of cross-project `blocks` and `related` edges. Each shows the linked node's title, type, project name, and a navigation link.

### Creating/Editing Nodes

- "New Project" opens a modal or inline form: title field + markdown editor pre-filled from template.
- "New Phase" / "New Task" from within the project detail view, contextually placed.
- Markdown editing: in-app textarea with preview toggle. No WYSIWYG — keep it simple.

## CLI (`brain-dump-cli`)

Installed as `bd` binary (short for brain-dump). Uses clap for argument parsing. Reads/writes `~/.brain-dump/brain-dump.db` directly via `brain-dump-core`.

The `--in` flag accepts either a ULID or a title (case-insensitive). If a title matches multiple nodes, the CLI prints matches and asks to disambiguate with the ID.

```
bd new project "OpsML v3"                          # Create project from template
bd new phase "Core Server" --in <project-id>       # Create phase under project
bd new task "Set up Axum" --in <phase-id>          # Create task under phase
bd list projects [--status active]                  # List projects, optionally filtered
bd list phases --in <project-id>                    # List phases in a project
bd list tasks --in <phase-id>                       # List tasks in a phase
bd show <id>                                        # Print node details + children + connections
bd edit <id> --title "New Title"                    # Update fields
bd edit <id> --description                          # Open $EDITOR for markdown
bd status <id> completed                            # Update status (logs to history)
bd link <source-id> blocks <target-id>              # Create dependency
bd link <source-id> related <target-id>             # Create soft link
bd unlink <source-id> <edge-type> <target-id>       # Remove specific edge type
bd delete <id>                                      # Delete node (cascades edges/tags)
bd tag <id> "rust"                                  # Add tag (creates tag if new)
bd untag <id> "rust"                                # Remove tag
```

**Output format**: Human-readable by default. `--json` flag for structured output (useful for Claude Code to parse).

**Error handling**: Errors go to stderr with non-zero exit code. With `--json`, errors are `{"error": "message"}`. Missing node IDs, invalid edge types, and hierarchy violations produce clear messages. On first run, creates `~/.brain-dump/` and initializes the DB automatically.

### Tauri Commands

Mirror the CLI surface for the SvelteKit frontend. Each maps to a function in `commands.rs`:

```
create_node(type, title, parent_id?) → Node
update_node(id, title?, description?, status?, sort_order?) → Node
delete_node(id) → ()
get_node(id) → Node (with children + edges)
list_nodes(type, parent_id?, status?, tag?) → Vec<Node>
create_edge(source_id, target_id, edge_type) → Edge
delete_edge(source_id, edge_type, target_id) → ()
add_tag(node_id, tag_name) → Tag
remove_tag(node_id, tag_name) → ()
list_tags() → Vec<Tag>
```

## Templates

Pre-filled markdown when creating new nodes.

**project.md:**
```markdown
## Goal
<!-- What are you trying to accomplish? -->

## Value
<!-- Why does this matter? What's the end outcome? -->

## Success Criteria
<!-- How do you know it's done? -->

## Context
<!-- Any background, constraints, or related work -->
```

**phase.md:**
```markdown
## Objective
<!-- What does this phase deliver? -->

## Scope
<!-- What's in and out of scope? -->
```

**task.md:**
```markdown
## Description
<!-- What needs to be done? -->

## Acceptance Criteria
<!-- How do you verify it's complete? -->
```

## Claude Code Integration (v1)

Claude Code interacts via the CLI. No special integration needed — Claude just runs `brain-dump` commands.

Typical workflow:
1. User: "Hey Claude, create a new project called 'Agent Evals'"
2. Claude runs: `bd new project "Agent Evals" --json`
3. Claude reads the template, helps fill in Goal/Value/Success Criteria
4. Claude runs: `bd edit <id> --title "Agent Evals"` and updates description
5. User: "Break this down into phases"
6. Claude creates phases and tasks via CLI, links dependencies

## In-App AI Chat (v2 — future)

Not in v1 scope. Will add a chat panel that calls the Anthropic API directly for brainstorming within the app. Requires API key configuration.

## Verification

### Build & Run
```bash
# Build everything
cargo build --workspace --all-features

# Run CLI
cargo run -p brain-dump-cli -- list projects

# Run desktop app
cd ui && pnpm install && pnpm build
cargo tauri dev
```

### Test
```bash
# Rust tests
cargo test --workspace --all-features

# Test CLI commands
bd new project "Test Project"
bd list projects
bd show <id>

# Verify SQLite DB
sqlite3 ~/.brain-dump/brain-dump.db ".tables"
sqlite3 ~/.brain-dump/brain-dump.db "SELECT * FROM nodes"
```

### UI Verification
- Open app → dashboard shows card grid
- Click "New Project" → form with template
- Click project card → split panel with overview + phases tab
- Resize window → cards reflow (1→2→3 columns)
- Test sidebar collapse/expand
