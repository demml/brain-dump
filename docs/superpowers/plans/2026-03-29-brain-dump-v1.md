# brain-dump v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri 2 desktop app for capturing ideas, breaking them into structured plans with phases/tasks, and visualizing cross-project dependencies — with a CLI for Claude Code integration.

**Architecture:** Rust workspace with 3 crates (core library, CLI binary, Tauri app) sharing a SQLite database. SvelteKit 2 + Svelte 5 frontend with Tailwind v4 and OpsML's phosphor-green CRT theme. No HTTP server — Tauri IPC for UI, direct SQLite for CLI.

**Tech Stack:** Rust (rusqlite, clap, serde, ulid), Tauri 2, SvelteKit 2, Svelte 5, Tailwind CSS v4, SQLite (WAL mode)

**Spec:** `docs/superpowers/specs/2026-03-29-brain-dump-design.md` (copy from `~/.claude/plans/starry-mixing-pony.md`)

**Reference projects:**
- OpsML UI theme: `/Users/stevenforrester/Documents/GitHub/opsml/crates/opsml_server/opsml_ui/opsml-theme.css`
- skill-check Tauri app: `/Users/stevenforrester/Documents/GitHub/skill-check/ui/`

---

## File Structure

### Rust workspace

```
brain-dump/
├── Cargo.toml                              # Workspace: members = ["crates/*"]
├── crates/
│   ├── brain-dump-core/
│   │   ├── Cargo.toml                      # rusqlite, serde, ulid, chrono
│   │   └── src/
│   │       ├── lib.rs                      # Re-exports: db, models, queries, templates
│   │       ├── db.rs                       # open_db(), migrate(), WAL setup
│   │       ├── models.rs                   # Node, Edge, Tag, NodeTag, StatusHistory, EdgeType, NodeType, NodeStatus
│   │       ├── queries.rs                  # All CRUD: create_node, get_node, list_nodes, update_node, delete_node, create_edge, delete_edge, add_tag, remove_tag, list_tags, get_children, get_edges, detect_cycle
│   │       └── templates.rs                # get_template(NodeType) → &str, uses include_str!()
│   ├── brain-dump-cli/
│   │   ├── Cargo.toml                      # clap, brain-dump-core, serde_json
│   │   └── src/
│   │       └── main.rs                     # CLI entry: subcommands new, list, show, edit, status, link, unlink, delete, tag, untag
│   └── brain-dump-app/
│       ├── Cargo.toml                      # tauri, brain-dump-core, serde, serde_json
│       ├── build.rs                        # tauri_build::build()
│       ├── src/
│       │   ├── main.rs                     # Tauri entry point
│       │   ├── lib.rs                      # AppState, run(), register commands
│       │   └── commands.rs                 # #[tauri::command] functions wrapping core queries
│       └── tauri.conf.json
```

### SvelteKit frontend

```
ui/
├── package.json
├── svelte.config.js                        # adapter-static
├── vite.config.ts                          # tailwindcss + sveltekit, port 1420
├── tsconfig.json
├── src/
│   ├── app.html                            # Shell with JetBrains Mono font
│   ├── app.css                             # Tailwind v4 @import + @theme + CRT theme
│   ├── routes/
│   │   ├── +layout.svelte                  # Sidebar + main content area
│   │   ├── +layout.ts                      # export const prerender = true; ssr = false
│   │   ├── +page.svelte                    # Dashboard: project card grid
│   │   └── project/
│   │       └── [id]/
│   │           └── +page.svelte            # Project detail: split panel + tabs
│   └── lib/
│       ├── tauri.ts                        # Typed invoke() wrappers
│       ├── types.ts                        # TS interfaces matching Rust models
│       └── components/
│           ├── Sidebar.svelte              # Collapsible icon sidebar
│           ├── ProjectCard.svelte          # Card for dashboard grid
│           ├── PhaseCard.svelte            # Expandable phase accordion
│           ├── TaskItem.svelte             # Task row with checkbox + badges
│           ├── MarkdownRenderer.svelte     # Render markdown to HTML
│           ├── ConnectionBadge.svelte      # Dependency/related link badge
│           ├── TagBadge.svelte             # Tag pill
│           ├── NodeForm.svelte             # Create/edit form with markdown editor
│           └── ProgressBar.svelte          # Segmented progress bar
```

### Templates (embedded at compile time)

```
templates/
├── project.md
├── phase.md
└── task.md
```

---

## Task 1: Project Scaffold + Workspace Setup

**Files:**
- Create: `Cargo.toml`, `crates/brain-dump-core/Cargo.toml`, `crates/brain-dump-core/src/lib.rs`, `crates/brain-dump-cli/Cargo.toml`, `crates/brain-dump-cli/src/main.rs`, `.gitignore`

- [ ] **Step 1: Initialize Cargo workspace**

Create `Cargo.toml`:
```toml
[workspace]
members = ["crates/*"]
resolver = "2"
```

Create `.gitignore`:
```
/target
ui/node_modules
ui/.svelte-kit
ui/build
.DS_Store
```

- [ ] **Step 2: Create brain-dump-core crate**

Create `crates/brain-dump-core/Cargo.toml`:
```toml
[package]
name = "brain-dump-core"
version = "0.1.0"
edition = "2024"

[dependencies]
rusqlite = { version = "0.39", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ulid = "1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"
```

Create `crates/brain-dump-core/src/lib.rs`:
```rust
pub mod db;
pub mod models;
pub mod queries;
pub mod templates;
```

- [ ] **Step 3: Create brain-dump-cli crate**

Create `crates/brain-dump-cli/Cargo.toml`:
```toml
[package]
name = "brain-dump-cli"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "bd"
path = "src/main.rs"

[dependencies]
brain-dump-core = { path = "../brain-dump-core" }
clap = { version = "4", features = ["derive"] }
serde_json = "1"
```

Create `crates/brain-dump-cli/src/main.rs`:
```rust
fn main() {
    println!("bd: brain-dump CLI");
}
```

- [ ] **Step 4: Verify workspace compiles**

Run: `cd /Users/stevenforrester/Documents/GitHub/brain-dump && cargo build --workspace --all-features`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml .gitignore crates/
git commit -m "feat: initialize Rust workspace with core and cli crates"
```

---

## Task 2: Core Models + DB Setup + Migrations

**Files:**
- Create: `crates/brain-dump-core/src/models.rs`, `crates/brain-dump-core/src/db.rs`

- [ ] **Step 1: Write models**

Create `crates/brain-dump-core/src/models.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Project,
    Phase,
    Task,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Phase => "phase",
            Self::Task => "task",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "project" => Ok(Self::Project),
            "phase" => Ok(Self::Phase),
            "task" => Ok(Self::Task),
            _ => Err(format!("invalid node type: {s}")),
        }
    }

    /// Returns the allowed child type for this node type (if any)
    pub fn child_type(&self) -> Option<NodeType> {
        match self {
            Self::Project => Some(Self::Phase),
            Self::Phase => Some(Self::Task),
            Self::Task => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Active,
    Completed,
    Archived,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("invalid status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeType {
    Parent,
    Blocks,
    Related,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Parent => "parent",
            Self::Blocks => "blocks",
            Self::Related => "related",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "parent" => Ok(Self::Parent),
            "blocks" => Ok(Self::Blocks),
            "related" => Ok(Self::Related),
            _ => Err(format!("invalid edge type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub description: String,
    pub status: NodeStatus,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusHistoryEntry {
    pub id: String,
    pub node_id: String,
    pub old_status: String,
    pub new_status: String,
    pub changed_at: String,
}

/// A node with its computed relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetail {
    pub node: Node,
    pub children: Vec<Node>,
    pub tags: Vec<Tag>,
    pub blocks: Vec<Edge>,
    pub blocked_by: Vec<Edge>,
    pub related: Vec<Edge>,
    pub progress: Option<Progress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub completed: usize,
    pub total: usize,
}
```

- [ ] **Step 2: Write DB initialization and migration**

Create `crates/brain-dump-core/src/db.rs`:
```rust
use std::path::PathBuf;

use rusqlite::Connection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Custom(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Returns the default DB path: ~/.brain-dump/brain-dump.db
pub fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".brain-dump").join("brain-dump.db")
}

/// Open (or create) the database, run migrations, enable WAL.
pub fn open_db(path: &std::path::Path) -> DbResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> DbResult<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL CHECK(type IN ('project','phase','task')),
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','completed','archived')),
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                target_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                edge_type TEXT NOT NULL CHECK(edge_type IN ('parent','blocks','related')),
                created_at TEXT NOT NULL,
                UNIQUE(source_id, target_id, edge_type)
            );
            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id, edge_type);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id, edge_type);

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#8ddb9f'
            );

            CREATE TABLE IF NOT EXISTS node_tags (
                node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY(node_id, tag_id)
            );

            CREATE TABLE IF NOT EXISTS status_history (
                id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                old_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                changed_at TEXT NOT NULL
            );

            PRAGMA user_version = 1;"
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_db_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        migrate(&conn).unwrap();

        let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0)).unwrap();
        assert_eq!(version, 1);

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"nodes".to_string()));
        assert!(tables.contains(&"edges".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"node_tags".to_string()));
        assert!(tables.contains(&"status_history".to_string()));
    }

    #[test]
    fn test_migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // Should not panic
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/stevenforrester/Documents/GitHub/brain-dump && cargo test --workspace --all-features`
Expected: 2 tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/brain-dump-core/src/
git commit -m "feat: add core models, DB initialization, and schema migration"
```

---

## Task 3: Templates

**Files:**
- Create: `templates/project.md`, `templates/phase.md`, `templates/task.md`, `crates/brain-dump-core/src/templates.rs`

- [ ] **Step 1: Create template files**

Create `templates/project.md`:
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

Create `templates/phase.md`:
```markdown
## Objective
<!-- What does this phase deliver? -->

## Scope
<!-- What's in and out of scope? -->
```

Create `templates/task.md`:
```markdown
## Description
<!-- What needs to be done? -->

## Acceptance Criteria
<!-- How do you verify it's complete? -->
```

- [ ] **Step 2: Write templates module**

Create `crates/brain-dump-core/src/templates.rs`:
```rust
use crate::models::NodeType;

const PROJECT_TEMPLATE: &str = include_str!("../../../templates/project.md");
const PHASE_TEMPLATE: &str = include_str!("../../../templates/phase.md");
const TASK_TEMPLATE: &str = include_str!("../../../templates/task.md");

pub fn get_template(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Project => PROJECT_TEMPLATE,
        NodeType::Phase => PHASE_TEMPLATE,
        NodeType::Task => TASK_TEMPLATE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_templates_are_nonempty() {
        assert!(get_template(NodeType::Project).contains("## Goal"));
        assert!(get_template(NodeType::Phase).contains("## Objective"));
        assert!(get_template(NodeType::Task).contains("## Description"));
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --workspace --all-features`
Expected: 3 tests pass (2 from db + 1 from templates)

- [ ] **Step 4: Commit**

```bash
git add templates/ crates/brain-dump-core/src/templates.rs
git commit -m "feat: add markdown templates for project, phase, and task nodes"
```

---

## Task 4: Core Queries — Node CRUD

**Files:**
- Create: `crates/brain-dump-core/src/queries.rs`

- [ ] **Step 1: Write failing tests for node CRUD**

Add to `crates/brain-dump-core/src/queries.rs`:
```rust
use rusqlite::{params, Connection};
use ulid::Ulid;

use crate::db::{DbError, DbResult};
use crate::models::*;
use crate::templates::get_template;

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Ulid::new().to_string()
}

pub fn create_node(
    conn: &Connection,
    node_type: NodeType,
    title: &str,
    parent_id: Option<&str>,
) -> DbResult<Node> {
    let id = new_id();
    let now = now_iso();
    let description = get_template(node_type).to_string();

    // If parent_id provided, validate hierarchy
    if let Some(pid) = parent_id {
        let parent = get_node(conn, pid)?;
        let expected_child = parent.node_type.child_type().ok_or_else(|| {
            DbError::Custom(format!("{} nodes cannot have children", parent.node_type.as_str()))
        })?;
        if expected_child != node_type {
            return Err(DbError::Custom(format!(
                "{} can only contain {} children, not {}",
                parent.node_type.as_str(),
                expected_child.as_str(),
                node_type.as_str()
            )));
        }
    }

    conn.execute(
        "INSERT INTO nodes (id, type, title, description, status, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'active', 0, ?5, ?5)",
        params![id, node_type.as_str(), title, description, now],
    )?;

    // Create parent edge if parent_id provided
    if let Some(pid) = parent_id {
        let edge_id = new_id();
        conn.execute(
            "INSERT INTO edges (id, source_id, target_id, edge_type, created_at)
             VALUES (?1, ?2, ?3, 'parent', ?4)",
            params![edge_id, pid, id, now],
        )?;
    }

    get_node(conn, &id)
}

pub fn get_node(conn: &Connection, id: &str) -> DbResult<Node> {
    conn.query_row(
        "SELECT id, type, title, description, status, sort_order, created_at, updated_at
         FROM nodes WHERE id = ?1",
        params![id],
        |row| {
            Ok(Node {
                id: row.get(0)?,
                node_type: NodeType::from_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                description: row.get(3)?,
                status: NodeStatus::from_str(&row.get::<_, String>(4)?).unwrap(),
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::Custom(format!("node not found: {id}")),
        other => DbError::Sqlite(other),
    })
}

pub fn list_nodes(
    conn: &Connection,
    node_type: NodeType,
    parent_id: Option<&str>,
    status: Option<NodeStatus>,
    tag: Option<&str>,
) -> DbResult<Vec<Node>> {
    let mut sql = String::from(
        "SELECT n.id, n.type, n.title, n.description, n.status, n.sort_order, n.created_at, n.updated_at
         FROM nodes n"
    );
    let mut conditions = vec!["n.type = :node_type".to_string()];

    if parent_id.is_some() {
        sql.push_str(" INNER JOIN edges e ON e.target_id = n.id AND e.edge_type = 'parent'");
        conditions.push("e.source_id = :parent_id".to_string());
    }

    if tag.is_some() {
        sql.push_str(" INNER JOIN node_tags nt ON nt.node_id = n.id INNER JOIN tags t ON t.id = nt.tag_id");
        conditions.push("t.name = :tag".to_string());
    }

    if status.is_some() {
        conditions.push("n.status = :status".to_string());
    }

    sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
    sql.push_str(" ORDER BY n.sort_order, n.created_at");

    let mut stmt = conn.prepare(&sql)?;

    // All params are parameterized — no string interpolation
    let mut param_values: Vec<(&str, Box<dyn rusqlite::types::ToSql>)> = Vec::new();
    param_values.push((":node_type", Box::new(node_type.as_str().to_string())));
    if let Some(pid) = parent_id {
        param_values.push((":parent_id", Box::new(pid.to_string())));
    }
    if let Some(t) = tag {
        param_values.push((":tag", Box::new(t.to_string())));
    }
    if let Some(s) = &status {
        param_values.push((":status", Box::new(s.as_str().to_string())));
    }

    let param_refs: Vec<(&str, &dyn rusqlite::types::ToSql)> =
        param_values.iter().map(|(k, v)| (*k, v.as_ref())).collect();

    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Node {
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?).unwrap(),
            title: row.get(2)?,
            description: row.get(3)?,
            status: NodeStatus::from_str(&row.get::<_, String>(4)?).unwrap(),
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row?);
    }
    Ok(nodes)
}

pub fn update_node(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<NodeStatus>,
    sort_order: Option<i32>,
) -> DbResult<Node> {
    let existing = get_node(conn, id)?;
    let now = now_iso();

    let new_title = title.unwrap_or(&existing.title);
    let new_desc = description.unwrap_or(&existing.description);
    let new_status = status.unwrap_or(existing.status);
    let new_order = sort_order.unwrap_or(existing.sort_order);

    conn.execute(
        "UPDATE nodes SET title = ?1, description = ?2, status = ?3, sort_order = ?4, updated_at = ?5
         WHERE id = ?6",
        params![new_title, new_desc, new_status.as_str(), new_order, now, id],
    )?;

    // Log status change
    if new_status != existing.status {
        let hist_id = new_id();
        conn.execute(
            "INSERT INTO status_history (id, node_id, old_status, new_status, changed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![hist_id, id, existing.status.as_str(), new_status.as_str(), now],
        )?;
    }

    get_node(conn, id)
}

pub fn delete_node(conn: &Connection, id: &str) -> DbResult<()> {
    // Verify node exists
    let _ = get_node(conn, id)?;
    // Recursively delete children first (CASCADE only removes edges, not child nodes)
    let children = get_children(conn, id)?;
    for child in &children {
        delete_node(conn, &child.id)?;
    }
    conn.execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        db::open_db_in_memory(&conn).unwrap();
        conn
    }

    // NOTE: We need a helper in db.rs for in-memory test DBs.
    // For now, tests will use the migrate function directly.
}
```

Actually — let me restructure. We need an in-memory helper in db.rs first.

Add to `crates/brain-dump-core/src/db.rs` (after the existing code, before tests):
```rust
/// Initialize an already-open in-memory connection (for tests).
pub fn init_in_memory(conn: &Connection) -> DbResult<()> {
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    migrate(conn)?;
    Ok(())
}
```

Then the test module in queries.rs:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init_in_memory(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_project() {
        let conn = setup();
        let node = create_node(&conn, NodeType::Project, "Test Project", None).unwrap();
        assert_eq!(node.title, "Test Project");
        assert_eq!(node.node_type, NodeType::Project);
        assert_eq!(node.status, NodeStatus::Active);
        assert!(node.description.contains("## Goal"));

        let fetched = get_node(&conn, &node.id).unwrap();
        assert_eq!(fetched.id, node.id);
    }

    #[test]
    fn test_create_phase_under_project() {
        let conn = setup();
        let project = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let phase = create_node(&conn, NodeType::Phase, "Phase 1", Some(&project.id)).unwrap();
        assert_eq!(phase.node_type, NodeType::Phase);

        let phases = list_nodes(&conn, NodeType::Phase, Some(&project.id), None, None).unwrap();
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].id, phase.id);
    }

    #[test]
    fn test_create_task_under_phase() {
        let conn = setup();
        let project = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let phase = create_node(&conn, NodeType::Phase, "Phase 1", Some(&project.id)).unwrap();
        let task = create_node(&conn, NodeType::Task, "Task 1", Some(&phase.id)).unwrap();
        assert_eq!(task.node_type, NodeType::Task);
    }

    #[test]
    fn test_hierarchy_enforcement() {
        let conn = setup();
        let project = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        // Can't add a task directly under a project
        let err = create_node(&conn, NodeType::Task, "T1", Some(&project.id));
        assert!(err.is_err());
    }

    #[test]
    fn test_update_node_status_logs_history() {
        let conn = setup();
        let node = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        update_node(&conn, &node.id, None, None, Some(NodeStatus::Completed), None).unwrap();

        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM status_history WHERE node_id = ?1",
            params![node.id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_delete_node_cascades() {
        let conn = setup();
        let project = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let phase = create_node(&conn, NodeType::Phase, "Phase 1", Some(&project.id)).unwrap();
        let _task = create_node(&conn, NodeType::Task, "Task 1", Some(&phase.id)).unwrap();

        delete_node(&conn, &project.id).unwrap();

        // Phase and task should also be gone (CASCADE)
        assert!(get_node(&conn, &phase.id).is_err());
    }

    #[test]
    fn test_list_projects_by_status() {
        let conn = setup();
        create_node(&conn, NodeType::Project, "Active", None).unwrap();
        let p2 = create_node(&conn, NodeType::Project, "Done", None).unwrap();
        update_node(&conn, &p2.id, None, None, Some(NodeStatus::Completed), None).unwrap();

        let active = list_nodes(&conn, NodeType::Project, None, Some(NodeStatus::Active), None).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "Active");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail (queries module doesn't exist yet)**

Run: `cargo test --workspace --all-features`
Expected: Compilation errors (that's fine — we're writing the test first, implementation is inline above)

- [ ] **Step 3: Ensure all code compiles and tests pass**

Since the tests and implementation are in the same file (queries.rs), write the full file as shown above and run:

Run: `cargo test --workspace --all-features`
Expected: All tests pass (db: 2, templates: 1, queries: 7)

- [ ] **Step 4: Commit**

```bash
git add crates/brain-dump-core/src/queries.rs crates/brain-dump-core/src/db.rs
git commit -m "feat: add node CRUD queries with hierarchy enforcement and status history"
```

---

## Task 5: Core Queries — Edges (Links, Dependencies, Cycle Detection)

**Files:**
- Modify: `crates/brain-dump-core/src/queries.rs`

- [ ] **Step 1: Add edge functions and tests**

Add to `crates/brain-dump-core/src/queries.rs`:
```rust
pub fn create_edge(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    edge_type: EdgeType,
) -> DbResult<Edge> {
    // Verify both nodes exist
    let _ = get_node(conn, source_id)?;
    let _ = get_node(conn, target_id)?;

    // Cycle detection for blocks edges
    if edge_type == EdgeType::Blocks {
        if has_path(conn, target_id, source_id, EdgeType::Blocks)? {
            return Err(DbError::Custom(
                "creating this edge would form a cycle in the dependency graph".to_string(),
            ));
        }
    }

    let id = new_id();
    let now = now_iso();

    conn.execute(
        "INSERT INTO edges (id, source_id, target_id, edge_type, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, source_id, target_id, edge_type.as_str(), now],
    )?;

    Ok(Edge { id, source_id: source_id.to_string(), target_id: target_id.to_string(), edge_type, created_at: now })
}

pub fn delete_edge(
    conn: &Connection,
    source_id: &str,
    edge_type: EdgeType,
    target_id: &str,
) -> DbResult<()> {
    let affected = conn.execute(
        "DELETE FROM edges WHERE source_id = ?1 AND target_id = ?2 AND edge_type = ?3",
        params![source_id, target_id, edge_type.as_str()],
    )?;
    if affected == 0 {
        return Err(DbError::Custom("edge not found".to_string()));
    }
    Ok(())
}

pub fn get_edges(conn: &Connection, node_id: &str) -> DbResult<Vec<Edge>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, target_id, edge_type, created_at FROM edges
         WHERE source_id = ?1 OR target_id = ?1"
    )?;
    let rows = stmt.query_map(params![node_id], |row| {
        Ok(Edge {
            id: row.get(0)?,
            source_id: row.get(1)?,
            target_id: row.get(2)?,
            edge_type: EdgeType::from_str(&row.get::<_, String>(3)?).unwrap(),
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::Sqlite)
}

/// BFS cycle detection: checks if there's a path from `from` to `to` via edges of `edge_type`.
fn has_path(conn: &Connection, from: &str, to: &str, edge_type: EdgeType) -> DbResult<bool> {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(from.to_string());

    let mut stmt = conn.prepare(
        "SELECT target_id FROM edges WHERE source_id = ?1 AND edge_type = ?2"
    )?;

    while let Some(current) = queue.pop_front() {
        if current == to {
            return Ok(true);
        }
        if !visited.insert(current.clone()) {
            continue;
        }
        let targets: Vec<String> = stmt
            .query_map(params![current, edge_type.as_str()], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        queue.extend(targets);
    }
    Ok(false)
}
```

Add tests:
```rust
    #[test]
    fn test_create_blocks_edge() {
        let conn = setup();
        let p1 = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let p2 = create_node(&conn, NodeType::Project, "P2", None).unwrap();
        let edge = create_edge(&conn, &p1.id, &p2.id, EdgeType::Blocks).unwrap();
        assert_eq!(edge.edge_type, EdgeType::Blocks);
    }

    #[test]
    fn test_cycle_detection() {
        let conn = setup();
        let p1 = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let p2 = create_node(&conn, NodeType::Project, "P2", None).unwrap();
        let p3 = create_node(&conn, NodeType::Project, "P3", None).unwrap();

        create_edge(&conn, &p1.id, &p2.id, EdgeType::Blocks).unwrap();
        create_edge(&conn, &p2.id, &p3.id, EdgeType::Blocks).unwrap();

        // p3 -> p1 would create a cycle
        let err = create_edge(&conn, &p3.id, &p1.id, EdgeType::Blocks);
        assert!(err.is_err());
    }

    #[test]
    fn test_related_edge() {
        let conn = setup();
        let p1 = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let p2 = create_node(&conn, NodeType::Project, "P2", None).unwrap();
        create_edge(&conn, &p1.id, &p2.id, EdgeType::Related).unwrap();

        let edges = get_edges(&conn, &p1.id).unwrap();
        let related: Vec<_> = edges.iter().filter(|e| e.edge_type == EdgeType::Related).collect();
        assert_eq!(related.len(), 1);
    }

    #[test]
    fn test_delete_edge() {
        let conn = setup();
        let p1 = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let p2 = create_node(&conn, NodeType::Project, "P2", None).unwrap();
        create_edge(&conn, &p1.id, &p2.id, EdgeType::Blocks).unwrap();
        delete_edge(&conn, &p1.id, EdgeType::Blocks, &p2.id).unwrap();

        let edges = get_edges(&conn, &p1.id).unwrap();
        let blocks: Vec<_> = edges.iter().filter(|e| e.edge_type == EdgeType::Blocks).collect();
        assert_eq!(blocks.len(), 0);
    }
```

- [ ] **Step 2: Run tests**

Run: `cargo test --workspace --all-features`
Expected: All tests pass (11+ tests)

- [ ] **Step 3: Commit**

```bash
git add crates/brain-dump-core/src/queries.rs
git commit -m "feat: add edge CRUD with BFS cycle detection for blocks dependencies"
```

---

## Task 6: Core Queries — Tags + NodeDetail

**Files:**
- Modify: `crates/brain-dump-core/src/queries.rs`

- [ ] **Step 1: Add tag functions and get_node_detail**

```rust
pub fn add_tag(conn: &Connection, node_id: &str, tag_name: &str) -> DbResult<Tag> {
    let _ = get_node(conn, node_id)?;

    // Upsert tag
    let tag_id = match conn.query_row(
        "SELECT id FROM tags WHERE name = ?1",
        params![tag_name],
        |row| row.get::<_, String>(0),
    ) {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let id = new_id();
            conn.execute(
                "INSERT INTO tags (id, name, color) VALUES (?1, ?2, '#8ddb9f')",
                params![id, tag_name],
            )?;
            id
        }
        Err(e) => return Err(DbError::Sqlite(e)),
    };

    conn.execute(
        "INSERT OR IGNORE INTO node_tags (node_id, tag_id) VALUES (?1, ?2)",
        params![node_id, tag_id],
    )?;

    Ok(Tag { id: tag_id, name: tag_name.to_string(), color: "#8ddb9f".to_string() })
}

pub fn remove_tag(conn: &Connection, node_id: &str, tag_name: &str) -> DbResult<()> {
    let affected = conn.execute(
        "DELETE FROM node_tags WHERE node_id = ?1 AND tag_id = (SELECT id FROM tags WHERE name = ?2)",
        params![node_id, tag_name],
    )?;
    if affected == 0 {
        return Err(DbError::Custom(format!("tag '{tag_name}' not found on node {node_id}")));
    }
    Ok(())
}

pub fn get_node_tags(conn: &Connection, node_id: &str) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t
         INNER JOIN node_tags nt ON nt.tag_id = t.id
         WHERE nt.node_id = ?1 ORDER BY t.name"
    )?;
    let rows = stmt.query_map(params![node_id], |row| {
        Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)? })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::Sqlite)
}

pub fn list_tags(conn: &Connection) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)? })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::Sqlite)
}

pub fn get_children(conn: &Connection, parent_id: &str) -> DbResult<Vec<Node>> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.type, n.title, n.description, n.status, n.sort_order, n.created_at, n.updated_at
         FROM nodes n INNER JOIN edges e ON e.target_id = n.id
         WHERE e.source_id = ?1 AND e.edge_type = 'parent'
         ORDER BY n.sort_order, n.created_at"
    )?;
    let rows = stmt.query_map(params![parent_id], |row| {
        Ok(Node {
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?).unwrap(),
            title: row.get(2)?,
            description: row.get(3)?,
            status: NodeStatus::from_str(&row.get::<_, String>(4)?).unwrap(),
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::Sqlite)
}

pub fn get_node_detail(conn: &Connection, id: &str) -> DbResult<NodeDetail> {
    let node = get_node(conn, id)?;
    let children = get_children(conn, id)?;
    let tags = get_node_tags(conn, id)?;
    let all_edges = get_edges(conn, id)?;

    let blocks: Vec<Edge> = all_edges.iter()
        .filter(|e| e.edge_type == EdgeType::Blocks && e.source_id == id)
        .cloned().collect();
    let blocked_by: Vec<Edge> = all_edges.iter()
        .filter(|e| e.edge_type == EdgeType::Blocks && e.target_id == id)
        .cloned().collect();
    let related: Vec<Edge> = all_edges.iter()
        .filter(|e| e.edge_type == EdgeType::Related)
        .cloned().collect();

    let progress = if !children.is_empty() {
        let completed = children.iter().filter(|c| c.status == NodeStatus::Completed).count();
        Some(Progress { completed, total: children.len() })
    } else {
        None
    };

    Ok(NodeDetail { node, children, tags, blocks, blocked_by, related, progress })
}

/// Resolve an ID-or-title to a node ID. Returns error if title matches multiple nodes.
pub fn resolve_node(conn: &Connection, id_or_title: &str) -> DbResult<String> {
    // Try as ID first
    if get_node(conn, id_or_title).is_ok() {
        return Ok(id_or_title.to_string());
    }

    // Try as title (case-insensitive)
    let mut stmt = conn.prepare(
        "SELECT id, title FROM nodes WHERE LOWER(title) = LOWER(?1)"
    )?;
    let matches: Vec<(String, String)> = stmt
        .query_map(params![id_or_title], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    match matches.len() {
        0 => Err(DbError::Custom(format!("no node found matching '{id_or_title}'"))),
        1 => Ok(matches[0].0.clone()),
        _ => {
            let options: Vec<String> = matches.iter()
                .map(|(id, title)| format!("  {id} — {title}"))
                .collect();
            Err(DbError::Custom(format!(
                "multiple nodes match '{id_or_title}':\n{}",
                options.join("\n")
            )))
        }
    }
}
```

Add tests:
```rust
    #[test]
    fn test_add_and_list_tags() {
        let conn = setup();
        let p = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        add_tag(&conn, &p.id, "rust").unwrap();
        add_tag(&conn, &p.id, "ml").unwrap();

        let tags = get_node_tags(&conn, &p.id).unwrap();
        assert_eq!(tags.len(), 2);

        let all_tags = list_tags(&conn).unwrap();
        assert_eq!(all_tags.len(), 2);
    }

    #[test]
    fn test_remove_tag() {
        let conn = setup();
        let p = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        add_tag(&conn, &p.id, "rust").unwrap();
        remove_tag(&conn, &p.id, "rust").unwrap();

        let tags = get_node_tags(&conn, &p.id).unwrap();
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_get_node_detail() {
        let conn = setup();
        let project = create_node(&conn, NodeType::Project, "P1", None).unwrap();
        let phase = create_node(&conn, NodeType::Phase, "Phase 1", Some(&project.id)).unwrap();
        create_node(&conn, NodeType::Task, "T1", Some(&phase.id)).unwrap();
        create_node(&conn, NodeType::Task, "T2", Some(&phase.id)).unwrap();
        add_tag(&conn, &project.id, "rust").unwrap();

        let detail = get_node_detail(&conn, &project.id).unwrap();
        assert_eq!(detail.children.len(), 1); // 1 phase
        assert_eq!(detail.tags.len(), 1);
        assert_eq!(detail.progress.unwrap().total, 1);
    }

    #[test]
    fn test_resolve_node_by_title() {
        let conn = setup();
        let p = create_node(&conn, NodeType::Project, "My Project", None).unwrap();
        let resolved = resolve_node(&conn, "my project").unwrap();
        assert_eq!(resolved, p.id);
    }
```

- [ ] **Step 2: Run tests**

Run: `cargo test --workspace --all-features`
Expected: All tests pass (15+ tests)

- [ ] **Step 3: Commit**

```bash
git add crates/brain-dump-core/src/queries.rs
git commit -m "feat: add tag CRUD, node detail, children queries, and title-based resolution"
```

---

## Task 7: CLI — Full Command Suite

**Files:**
- Modify: `crates/brain-dump-cli/src/main.rs`

- [ ] **Step 1: Implement the CLI with all subcommands**

Replace `crates/brain-dump-cli/src/main.rs`:
```rust
use std::process;

use brain_dump_core::db;
use brain_dump_core::models::*;
use brain_dump_core::queries;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bd", about = "brain-dump: capture ideas, plan projects")]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new node
    New {
        #[command(subcommand)]
        kind: NewKind,
    },
    /// List nodes
    List {
        #[command(subcommand)]
        kind: ListKind,
    },
    /// Show node details
    Show { id: String },
    /// Edit a node
    Edit {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: bool,
    },
    /// Update node status
    Status { id: String, status: String },
    /// Create a link between nodes
    Link {
        source: String,
        edge_type: String,
        target: String,
    },
    /// Remove a link between nodes
    Unlink {
        source: String,
        edge_type: String,
        target: String,
    },
    /// Delete a node
    Delete { id: String },
    /// Add a tag to a node
    Tag { id: String, name: String },
    /// Remove a tag from a node
    Untag { id: String, name: String },
}

#[derive(Subcommand)]
enum NewKind {
    Project { title: String },
    Phase {
        title: String,
        #[arg(long, alias = "in")]
        parent: String,
    },
    Task {
        title: String,
        #[arg(long, alias = "in")]
        parent: String,
    },
}

#[derive(Subcommand)]
enum ListKind {
    Projects {
        #[arg(long)]
        status: Option<String>,
    },
    Phases {
        #[arg(long, alias = "in")]
        parent: String,
    },
    Tasks {
        #[arg(long, alias = "in")]
        parent: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let db_path = db::default_db_path();

    let conn = match db::open_db(&db_path) {
        Ok(c) => c,
        Err(e) => {
            err_exit(&cli, &format!("failed to open database: {e}"));
        }
    };

    let result = run(&cli, &conn);

    if let Err(e) = result {
        err_exit(&cli, &e.to_string());
    }
}

fn run(cli: &Cli, conn: &rusqlite::Connection) -> Result<(), brain_dump_core::db::DbError> {
    match &cli.command {
        Commands::New { kind } => {
            let node = match kind {
                NewKind::Project { title } => {
                    queries::create_node(conn, NodeType::Project, title, None)?
                }
                NewKind::Phase { title, parent } => {
                    let pid = queries::resolve_node(conn, parent)?;
                    queries::create_node(conn, NodeType::Phase, title, Some(&pid))?
                }
                NewKind::Task { title, parent } => {
                    let pid = queries::resolve_node(conn, parent)?;
                    queries::create_node(conn, NodeType::Task, title, Some(&pid))?
                }
            };
            output_node(cli, &node);
        }

        Commands::List { kind } => {
            let nodes = match kind {
                ListKind::Projects { status } => {
                    let s = status.as_deref().map(|s| NodeStatus::from_str(s).unwrap());
                    queries::list_nodes(conn, NodeType::Project, None, s, None)?
                }
                ListKind::Phases { parent } => {
                    let pid = queries::resolve_node(conn, parent)?;
                    queries::list_nodes(conn, NodeType::Phase, Some(&pid), None, None)?
                }
                ListKind::Tasks { parent } => {
                    let pid = queries::resolve_node(conn, parent)?;
                    queries::list_nodes(conn, NodeType::Task, Some(&pid), None, None)?
                }
            };
            output_nodes(cli, &nodes);
        }

        Commands::Show { id } => {
            let resolved = queries::resolve_node(conn, id)?;
            let detail = queries::get_node_detail(conn, &resolved)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&detail).unwrap());
            } else {
                print_detail(&detail);
            }
        }

        Commands::Edit { id, title, description } => {
            let resolved = queries::resolve_node(conn, id)?;
            let new_desc = if *description {
                let existing = queries::get_node(conn, &resolved)?;
                let edited = edit_in_editor(&existing.description)?;
                Some(edited)
            } else {
                None
            };
            let node = queries::update_node(
                conn,
                &resolved,
                title.as_deref(),
                new_desc.as_deref(),
                None,
                None,
            )?;
            output_node(cli, &node);
        }

        Commands::Status { id, status } => {
            let resolved = queries::resolve_node(conn, id)?;
            let s = NodeStatus::from_str(status)
                .map_err(|e| brain_dump_core::db::DbError::Custom(e))?;
            let node = queries::update_node(conn, &resolved, None, None, Some(s), None)?;
            output_node(cli, &node);
        }

        Commands::Link { source, edge_type, target } => {
            let src = queries::resolve_node(conn, source)?;
            let tgt = queries::resolve_node(conn, target)?;
            let et = EdgeType::from_str(edge_type)
                .map_err(|e| brain_dump_core::db::DbError::Custom(e))?;
            let edge = queries::create_edge(conn, &src, &tgt, et)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&edge).unwrap());
            } else {
                println!("Linked {} --[{}]--> {}", source, edge_type, target);
            }
        }

        Commands::Unlink { source, edge_type, target } => {
            let src = queries::resolve_node(conn, source)?;
            let tgt = queries::resolve_node(conn, target)?;
            let et = EdgeType::from_str(edge_type)
                .map_err(|e| brain_dump_core::db::DbError::Custom(e))?;
            queries::delete_edge(conn, &src, et, &tgt)?;
            println!("Unlinked {} --[{}]--> {}", source, edge_type, target);
        }

        Commands::Delete { id } => {
            let resolved = queries::resolve_node(conn, id)?;
            let node = queries::get_node(conn, &resolved)?;
            queries::delete_node(conn, &resolved)?;
            println!("Deleted {} '{}'", node.node_type.as_str(), node.title);
        }

        Commands::Tag { id, name } => {
            let resolved = queries::resolve_node(conn, id)?;
            queries::add_tag(conn, &resolved, name)?;
            println!("Tagged with '{name}'");
        }

        Commands::Untag { id, name } => {
            let resolved = queries::resolve_node(conn, id)?;
            queries::remove_tag(conn, &resolved, name)?;
            println!("Removed tag '{name}'");
        }
    }
    Ok(())
}

fn output_node(cli: &Cli, node: &Node) {
    if cli.json {
        println!("{}", serde_json::to_string_pretty(node).unwrap());
    } else {
        println!("[{}] {} — {} ({})", node.node_type.as_str(), node.id, node.title, node.status.as_str());
    }
}

fn output_nodes(cli: &Cli, nodes: &[Node]) {
    if cli.json {
        println!("{}", serde_json::to_string_pretty(nodes).unwrap());
    } else {
        if nodes.is_empty() {
            println!("No results.");
            return;
        }
        for node in nodes {
            println!("[{}] {} — {} ({})", node.node_type.as_str(), node.id, node.title, node.status.as_str());
        }
    }
}

fn print_detail(detail: &NodeDetail) {
    let n = &detail.node;
    println!("=== {} ===", n.title);
    println!("Type: {}  Status: {}  ID: {}", n.node_type.as_str(), n.status.as_str(), n.id);

    if !detail.tags.is_empty() {
        let tag_names: Vec<&str> = detail.tags.iter().map(|t| t.name.as_str()).collect();
        println!("Tags: {}", tag_names.join(", "));
    }

    if let Some(ref p) = detail.progress {
        println!("Progress: {}/{}", p.completed, p.total);
    }

    println!("\n{}", n.description);

    if !detail.children.is_empty() {
        println!("\n--- Children ---");
        for child in &detail.children {
            let status_mark = if child.status == NodeStatus::Completed { "✓" } else { "○" };
            println!("  {status_mark} [{}] {} — {}", child.node_type.as_str(), child.id, child.title);
        }
    }

    if !detail.blocks.is_empty() || !detail.blocked_by.is_empty() || !detail.related.is_empty() {
        println!("\n--- Connections ---");
        for e in &detail.blocks {
            println!("  → blocks {}", e.target_id);
        }
        for e in &detail.blocked_by {
            println!("  ← blocked by {}", e.source_id);
        }
        for e in &detail.related {
            let other = if e.source_id == n.id { &e.target_id } else { &e.source_id };
            println!("  ↔ related to {other}");
        }
    }
}

fn edit_in_editor(content: &str) -> Result<String, brain_dump_core::db::DbError> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let tmp = std::env::temp_dir().join("bd-edit.md");
    std::fs::write(&tmp, content).map_err(|e| brain_dump_core::db::DbError::Io(e))?;

    let status = std::process::Command::new(&editor)
        .arg(&tmp)
        .status()
        .map_err(|e| brain_dump_core::db::DbError::Io(e))?;

    if !status.success() {
        return Err(brain_dump_core::db::DbError::Custom("editor exited with error".to_string()));
    }

    std::fs::read_to_string(&tmp).map_err(|e| brain_dump_core::db::DbError::Io(e))
}

fn err_exit(cli: &Cli, msg: &str) -> ! {
    if cli.json {
        eprintln!("{}", serde_json::json!({"error": msg}));
    } else {
        eprintln!("error: {msg}");
    }
    process::exit(1);
}
```

- [ ] **Step 2: Build and verify CLI compiles**

Run: `cargo build --workspace --all-features`
Expected: Compiles successfully

- [ ] **Step 3: Smoke test the CLI**

Run:
```bash
cargo run -p brain-dump-cli -- new project "Test Project" --json
cargo run -p brain-dump-cli -- list projects
```
Expected: JSON output with node details, then a list showing the project

- [ ] **Step 4: Commit**

```bash
git add crates/brain-dump-cli/
git commit -m "feat: implement full CLI with all subcommands (new, list, show, edit, status, link, delete, tag)"
```

---

## Task 8: Tauri App Scaffold

**Files:**
- Create: `crates/brain-dump-app/Cargo.toml`, `crates/brain-dump-app/build.rs`, `crates/brain-dump-app/src/main.rs`, `crates/brain-dump-app/src/lib.rs`, `crates/brain-dump-app/src/commands.rs`, `crates/brain-dump-app/tauri.conf.json`

- [ ] **Step 1: Create Tauri app crate**

Create `crates/brain-dump-app/Cargo.toml`:
```toml
[package]
name = "brain-dump-app"
version = "0.1.0"
edition = "2024"

[lib]
name = "brain_dump_app_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
brain-dump-core = { path = "../brain-dump-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Create `crates/brain-dump-app/build.rs`:
```rust
fn main() {
    tauri_build::build()
}
```

Create `crates/brain-dump-app/tauri.conf.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/nicholasgasior/tauri-schema/refs/heads/main/tauri.conf.json",
  "productName": "brain-dump",
  "version": "0.1.0",
  "identifier": "com.braindump.app",
  "build": {
    "frontendDist": "../../../ui/build",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "cd ../../../ui && pnpm dev",
    "beforeBuildCommand": "cd ../../../ui && pnpm build"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "brain-dump",
        "width": 1280,
        "height": 900
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

- [ ] **Step 2: Create commands.rs with Tauri invoke handlers**

Create `crates/brain-dump-app/src/commands.rs`:
```rust
use std::sync::Mutex;

use brain_dump_core::db::DbResult;
use brain_dump_core::models::*;
use brain_dump_core::queries;
use rusqlite::Connection;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[tauri::command]
pub fn create_node(
    state: State<AppState>,
    node_type: String,
    title: String,
    parent_id: Option<String>,
) -> Result<Node, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let nt = NodeType::from_str(&node_type).map_err(|e| e.to_string())?;
    queries::create_node(&conn, nt, &title, parent_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_node(
    state: State<AppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    sort_order: Option<i32>,
) -> Result<Node, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let s = match status.as_deref() {
        Some(s) => Some(NodeStatus::from_str(s).map_err(|e| e.to_string())?),
        None => None,
    };
    queries::update_node(&conn, &id, title.as_deref(), description.as_deref(), s, sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_node(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_node(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_node(state: State<AppState>, id: String) -> Result<NodeDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_node_detail(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_nodes(
    state: State<AppState>,
    node_type: String,
    parent_id: Option<String>,
    status: Option<String>,
    tag: Option<String>,
) -> Result<Vec<Node>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let nt = NodeType::from_str(&node_type).map_err(|e| e.to_string())?;
    let s = match status.as_deref() {
        Some(s) => Some(NodeStatus::from_str(s).map_err(|e| e.to_string())?),
        None => None,
    };
    queries::list_nodes(&conn, nt, parent_id.as_deref(), s, tag.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_edge(
    state: State<AppState>,
    source_id: String,
    target_id: String,
    edge_type: String,
) -> Result<Edge, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let et = EdgeType::from_str(&edge_type).map_err(|e| e.to_string())?;
    queries::create_edge(&conn, &source_id, &target_id, et)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_edge(
    state: State<AppState>,
    source_id: String,
    edge_type: String,
    target_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let et = EdgeType::from_str(&edge_type).map_err(|e| e.to_string())?;
    queries::delete_edge(&conn, &source_id, et, &target_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag(
    state: State<AppState>,
    node_id: String,
    tag_name: String,
) -> Result<Tag, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_tag(&conn, &node_id, &tag_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag(
    state: State<AppState>,
    node_id: String,
    tag_name: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::remove_tag(&conn, &node_id, &tag_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tags(state: State<AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_tags(&conn).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: Create lib.rs and main.rs**

Create `crates/brain-dump-app/src/lib.rs`:
```rust
use std::sync::Mutex;

use brain_dump_core::db;

mod commands;
use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = db::default_db_path();
    let conn = db::open_db(&db_path).expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState { db: Mutex::new(conn) })
        .invoke_handler(tauri::generate_handler![
            commands::create_node,
            commands::update_node,
            commands::delete_node,
            commands::get_node,
            commands::list_nodes,
            commands::create_edge,
            commands::delete_edge,
            commands::add_tag,
            commands::remove_tag,
            commands::list_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

Create `crates/brain-dump-app/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    brain_dump_app_lib::run()
}
```

- [ ] **Step 4: Build (may need Tauri CLI installed)**

Run: `cargo build -p brain-dump-app --all-features`
Expected: Compiles (may warn about missing frontend dist, that's fine for now)

- [ ] **Step 5: Commit**

```bash
git add crates/brain-dump-app/
git commit -m "feat: add Tauri 2 app with all invoke command handlers"
```

---

## Task 9: SvelteKit Frontend Scaffold

**Files:**
- Create: `ui/package.json`, `ui/svelte.config.js`, `ui/vite.config.ts`, `ui/tsconfig.json`, `ui/src/app.html`, `ui/src/app.css`, `ui/src/routes/+layout.svelte`, `ui/src/routes/+layout.ts`, `ui/src/routes/+page.svelte`, `ui/src/lib/types.ts`, `ui/src/lib/tauri.ts`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "brain-dump-ui",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@tauri-apps/api": "^2"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3",
    "@sveltejs/kit": "^2",
    "@sveltejs/vite-plugin-svelte": "^5",
    "@tailwindcss/vite": "^4",
    "svelte": "^5",
    "tailwindcss": "^4",
    "typescript": "^5",
    "vite": "^6",
    "marked": "^15"
  }
}
```

- [ ] **Step 2: Create config files**

Create `ui/svelte.config.js`:
```javascript
import adapter from "@sveltejs/adapter-static";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter(),
  },
};

export default config;
```

Create `ui/vite.config.ts`:
```typescript
import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
  },
});
```

Create `ui/tsconfig.json`:
```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler"
  }
}
```

- [ ] **Step 3: Create app.html shell**

Create `ui/src/app.html`:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@300;400;500;600;700&display=swap" rel="stylesheet" />
    <title>brain-dump</title>
    %sveltekit.head%
  </head>
  <body>
    <div id="app">%sveltekit.body%</div>
  </body>
</html>
```

- [ ] **Step 4: Create the CRT phosphor-green theme**

Create `ui/src/app.css`. This is the core theme — port from OpsML's dark mode:
```css
@import "tailwindcss";

@theme {
  /* Phosphor green palette */
  --color-bg: oklch(5.5% 0.003 150);
  --color-bg-card: oklch(15% 0.004 150);
  --color-bg-card-hover: oklch(18% 0.006 150);
  --color-bg-input: oklch(10% 0.004 150);
  --color-border: oklch(40% 0.08 150 / 0.4);
  --color-border-bright: oklch(55% 0.10 150 / 0.6);
  --color-phosphor: oklch(82% 0.14 152);
  --color-phosphor-dim: oklch(65% 0.10 150);
  --color-phosphor-bright: oklch(88% 0.16 150);
  --color-phosphor-muted: oklch(50% 0.08 150);
  --color-danger: oklch(65% 0.20 25);
  --color-warning: oklch(75% 0.15 80);
  --color-success: oklch(75% 0.15 150);

  /* Neo-brutalist shadows */
  --shadow-hard: 3px 3px 0 0 oklch(40% 0.08 150 / 0.3);
  --shadow-glow: 0 0 12px 2px oklch(65% 0.14 150 / 0.15);

  /* Font */
  --font-family-mono: "JetBrains Mono", monospace;
}

/* Base styles */
body {
  background-color: var(--color-bg);
  color: var(--color-phosphor);
  font-family: var(--font-family-mono);
  margin: 0;
  min-height: 100vh;
}

/* Scanline CRT overlay */
body::after {
  content: "";
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9999;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    oklch(0% 0 0 / 0.06) 2px,
    oklch(0% 0 0 / 0.06) 4px
  );
}

/* Vignette */
body::before {
  content: "";
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9998;
  background: radial-gradient(
    ellipse at center,
    transparent 60%,
    oklch(0% 0 0 / 0.3) 100%
  );
}

/* Phosphor bloom on text */
h1, h2, h3, h4, strong {
  text-shadow: 0 0 1px oklch(75% 0.08 150 / 0.12);
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: var(--color-bg);
}
::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-bright);
}
```

- [ ] **Step 5: Create types.ts and tauri.ts**

Create `ui/src/lib/types.ts`:
```typescript
export interface Node {
  id: string;
  node_type: "project" | "phase" | "task";
  title: string;
  description: string;
  status: "active" | "completed" | "archived";
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Edge {
  id: string;
  source_id: string;
  target_id: string;
  edge_type: "parent" | "blocks" | "related";
  created_at: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string;
}

export interface Progress {
  completed: number;
  total: number;
}

export interface NodeDetail {
  node: Node;
  children: Node[];
  tags: Tag[];
  blocks: Edge[];
  blocked_by: Edge[];
  related: Edge[];
  progress: Progress | null;
}
```

Create `ui/src/lib/tauri.ts`:
```typescript
import { invoke } from "@tauri-apps/api/core";
import type { Node, NodeDetail, Edge, Tag } from "./types";

export async function createNode(
  nodeType: string,
  title: string,
  parentId?: string,
): Promise<Node> {
  return invoke("create_node", { nodeType, title, parentId: parentId ?? null });
}

export async function updateNode(
  id: string,
  title?: string,
  description?: string,
  status?: string,
  sortOrder?: number,
): Promise<Node> {
  return invoke("update_node", {
    id,
    title: title ?? null,
    description: description ?? null,
    status: status ?? null,
    sortOrder: sortOrder ?? null,
  });
}

export async function deleteNode(id: string): Promise<void> {
  return invoke("delete_node", { id });
}

export async function getNode(id: string): Promise<NodeDetail> {
  return invoke("get_node", { id });
}

export async function listNodes(
  nodeType: string,
  parentId?: string,
  status?: string,
  tag?: string,
): Promise<Node[]> {
  return invoke("list_nodes", {
    nodeType,
    parentId: parentId ?? null,
    status: status ?? null,
    tag: tag ?? null,
  });
}

export async function createEdge(
  sourceId: string,
  targetId: string,
  edgeType: string,
): Promise<Edge> {
  return invoke("create_edge", { sourceId, targetId, edgeType });
}

export async function deleteEdge(
  sourceId: string,
  edgeType: string,
  targetId: string,
): Promise<void> {
  return invoke("delete_edge", { sourceId, edgeType, targetId });
}

export async function addTag(nodeId: string, tagName: string): Promise<Tag> {
  return invoke("add_tag", { nodeId, tagName });
}

export async function removeTag(nodeId: string, tagName: string): Promise<void> {
  return invoke("remove_tag", { nodeId, tagName });
}

export async function listTags(): Promise<Tag[]> {
  return invoke("list_tags");
}
```

- [ ] **Step 6: Create layout and placeholder page**

Create `ui/src/routes/+layout.ts`:
```typescript
export const prerender = true;
export const ssr = false;
```

Create `ui/src/routes/+layout.svelte`:
```svelte
<script>
  import "../app.css";
  let { children } = $props();
</script>

<div class="flex min-h-screen">
  <!-- Sidebar placeholder -->
  <aside class="w-16 border-r-2 border-[var(--color-border)] bg-[var(--color-bg-card)] flex flex-col items-center py-4 gap-4">
    <div class="text-[var(--color-phosphor)] font-bold text-lg">bd</div>
  </aside>

  <!-- Main content -->
  <main class="flex-1 p-6 overflow-auto">
    {@render children()}
  </main>
</div>
```

Create `ui/src/routes/+page.svelte`:
```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { listNodes } from "$lib/tauri";
  import type { Node } from "$lib/types";

  let projects = $state<Node[]>([]);
  let loading = $state(true);

  onMount(async () => {
    projects = await listNodes("project");
    loading = false;
  });
</script>

<div class="max-w-6xl mx-auto">
  <div class="flex items-center justify-between mb-8">
    <h1 class="text-2xl font-bold text-[var(--color-phosphor-bright)]">brain-dump</h1>
    <button
      class="px-4 py-2 border-2 border-[var(--color-border-bright)] bg-[var(--color-bg-card)] text-[var(--color-phosphor)] hover:bg-[var(--color-bg-card-hover)] transition-colors"
      style="box-shadow: var(--shadow-hard);"
    >
      + New Project
    </button>
  </div>

  {#if loading}
    <p class="text-[var(--color-phosphor-dim)]">Loading...</p>
  {:else if projects.length === 0}
    <div class="border-2 border-dashed border-[var(--color-border)] p-8 text-center text-[var(--color-phosphor-dim)]">
      <p>No projects yet. Create one to get started.</p>
      <p class="text-sm mt-2">Or run: <code class="text-[var(--color-phosphor)]">bd new project "My Idea"</code></p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each projects as project}
        <a
          href="/project/{project.id}"
          class="block p-4 border-2 border-[var(--color-border)] bg-[var(--color-bg-card)] hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-none transition-all"
          style="box-shadow: var(--shadow-hard);"
        >
          <h3 class="font-bold text-[var(--color-phosphor-bright)] mb-2">{project.title}</h3>
          <p class="text-sm text-[var(--color-phosphor-dim)] line-clamp-2">{project.description.slice(0, 100)}</p>
          <div class="mt-3 flex items-center gap-2 text-xs text-[var(--color-phosphor-muted)]">
            <span>{project.status}</span>
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>
```

- [ ] **Step 7: Install dependencies and verify build**

Run:
```bash
cd /Users/stevenforrester/Documents/GitHub/brain-dump/ui && pnpm install && pnpm build
```
Expected: SvelteKit builds to `ui/build/` directory

- [ ] **Step 8: Commit**

```bash
git add ui/
git commit -m "feat: scaffold SvelteKit frontend with CRT theme, types, Tauri invoke wrappers, and dashboard placeholder"
```

---

## Task 10: UI Components — Sidebar, ProjectCard, ProgressBar, TagBadge

**Files:**
- Create: `ui/src/lib/components/Sidebar.svelte`, `ui/src/lib/components/ProjectCard.svelte`, `ui/src/lib/components/ProgressBar.svelte`, `ui/src/lib/components/TagBadge.svelte`
- Modify: `ui/src/routes/+layout.svelte`, `ui/src/routes/+page.svelte`

- [ ] **Step 1: Create Sidebar component**

Create `ui/src/lib/components/Sidebar.svelte` — collapsible icon sidebar matching OpsML pattern: collapsed = 4rem, expanded = 16rem, hover to expand, pin toggle. Nav items: Home, Projects, Tags, Settings. Use inline SVG icons or simple text icons for now.

- [ ] **Step 2: Create ProjectCard component**

Create `ui/src/lib/components/ProjectCard.svelte` — accepts a Node prop + tags + progress. Renders: title, description snippet (truncated), status badge, progress bar, tag badges. Neo-brutalist card with hard shadow, hover translate effect.

- [ ] **Step 3: Create ProgressBar and TagBadge**

Create `ui/src/lib/components/ProgressBar.svelte` — segmented bar showing completed/total. Phosphor green for completed segments, dim for incomplete.

Create `ui/src/lib/components/TagBadge.svelte` — small pill with tag name and color dot.

- [ ] **Step 4: Wire into dashboard page**

Update `ui/src/routes/+layout.svelte` to use Sidebar component.
Update `ui/src/routes/+page.svelte` to use ProjectCard with real data from Tauri.

- [ ] **Step 5: Build and verify**

Run: `cd /Users/stevenforrester/Documents/GitHub/brain-dump/ui && pnpm build`
Expected: Builds successfully

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/components/ ui/src/routes/
git commit -m "feat: add Sidebar, ProjectCard, ProgressBar, and TagBadge components"
```

---

## Task 11: UI — Project Detail Page (Split Panel + Tabs)

**Files:**
- Create: `ui/src/routes/project/[id]/+page.svelte`, `ui/src/lib/components/PhaseCard.svelte`, `ui/src/lib/components/TaskItem.svelte`, `ui/src/lib/components/MarkdownRenderer.svelte`, `ui/src/lib/components/ConnectionBadge.svelte`

- [ ] **Step 1: Create MarkdownRenderer**

Create `ui/src/lib/components/MarkdownRenderer.svelte` — accepts markdown string prop, renders to HTML using `marked` library. Style rendered HTML with phosphor-green theme (headings, lists, code blocks, links).

- [ ] **Step 2: Create PhaseCard and TaskItem**

Create `ui/src/lib/components/PhaseCard.svelte` — expandable accordion card. Shows phase title, segmented progress bar, task count. When expanded, renders child TaskItem components.

Create `ui/src/lib/components/TaskItem.svelte` — single task row with checkbox (toggles status), title, dependency badges (ConnectionBadge).

- [ ] **Step 3: Create ConnectionBadge**

Create `ui/src/lib/components/ConnectionBadge.svelte` — small badge showing "blocks: Node Title" (red tint) or "related: Node Title" (blue tint). Clickable to navigate.

- [ ] **Step 4: Create project detail page**

Create `ui/src/routes/project/[id]/+page.svelte`:
- Load node detail via `getNode(id)` on mount
- Split panel: left (40%) = MarkdownRenderer with description, right (60%) = tabbed (Phases & Tasks / Connections)
- Back button, title, status badge, edit/archive actions in header
- Phases tab: list of PhaseCard components
- Connections tab: list of ConnectionBadge for blocks/blocked_by/related edges

- [ ] **Step 5: Build and verify**

Run: `cd /Users/stevenforrester/Documents/GitHub/brain-dump/ui && pnpm build`
Expected: Builds successfully

- [ ] **Step 6: Commit**

```bash
git add ui/src/routes/project/ ui/src/lib/components/
git commit -m "feat: add project detail page with split panel, phase accordion, and connection badges"
```

---

## Task 12: UI — Create/Edit Node Forms

**Files:**
- Create: `ui/src/lib/components/NodeForm.svelte`
- Modify: `ui/src/routes/+page.svelte`, `ui/src/routes/project/[id]/+page.svelte`

- [ ] **Step 1: Create NodeForm component**

Create `ui/src/lib/components/NodeForm.svelte` — modal/overlay form. Props: node type, optional existing node (for edit). Fields: title input, markdown textarea with preview toggle. Pre-fills description from template for new nodes. Save and Cancel buttons.

- [ ] **Step 2: Wire "New Project" button on dashboard**

Update `ui/src/routes/+page.svelte` — clicking "New Project" opens NodeForm as modal. On save, calls `createNode` and refreshes the project list.

- [ ] **Step 3: Wire "New Phase" / "New Task" + "Edit" in project detail**

Update `ui/src/routes/project/[id]/+page.svelte`:
- "New Phase" button below phases list → opens NodeForm with type=phase, parent=project.id
- "New Task" button inside expanded PhaseCard → opens NodeForm with type=task, parent=phase.id
- "Edit" action on header → opens NodeForm with existing node data

- [ ] **Step 4: Build and verify**

Run: `cd /Users/stevenforrester/Documents/GitHub/brain-dump/ui && pnpm build`
Expected: Builds successfully

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/components/NodeForm.svelte ui/src/routes/
git commit -m "feat: add NodeForm component with create/edit flows for all node types"
```

---

## Task 13: Integration Test — End to End

**Files:** None new — this is a verification task.

- [ ] **Step 1: Build everything**

```bash
cd /Users/stevenforrester/Documents/GitHub/brain-dump
cargo build --workspace --all-features
cd ui && pnpm install && pnpm build && cd ..
```

- [ ] **Step 2: Run all Rust tests**

```bash
cargo test --workspace --all-features
```
Expected: All tests pass

- [ ] **Step 3: CLI smoke test**

```bash
cargo run -p brain-dump-cli -- new project "Integration Test"
cargo run -p brain-dump-cli -- list projects
# Capture the project ID from output
cargo run -p brain-dump-cli -- new phase "Phase 1" --parent "Integration Test"
cargo run -p brain-dump-cli -- list phases --parent "Integration Test"
cargo run -p brain-dump-cli -- tag "Integration Test" "test"
cargo run -p brain-dump-cli -- show "Integration Test" --json
```
Expected: All commands succeed, show output shows full detail with phase and tag

- [ ] **Step 4: Launch Tauri app**

```bash
cd /Users/stevenforrester/Documents/GitHub/brain-dump/crates/brain-dump-app
cargo tauri dev
```
Expected: App window opens, shows dashboard with the "Integration Test" project card created via CLI

- [ ] **Step 5: UI interaction test**

In the running app:
- Verify project card shows on dashboard
- Click project card → detail page loads with split panel
- Verify markdown rendered on left, phases on right
- Create a new project via the UI form

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "chore: verify end-to-end integration — CLI and Tauri app both working"
```

---

## Summary

| Task | What it delivers | Key files |
|------|-----------------|-----------|
| 1 | Cargo workspace compiles | `Cargo.toml`, crate stubs |
| 2 | DB schema + migrations | `db.rs`, `models.rs` |
| 3 | Markdown templates | `templates.rs`, `templates/*.md` |
| 4 | Node CRUD with hierarchy | `queries.rs` |
| 5 | Edge CRUD with cycle detection | `queries.rs` |
| 6 | Tags + NodeDetail | `queries.rs` |
| 7 | Full CLI | `brain-dump-cli/src/main.rs` |
| 8 | Tauri app shell | `brain-dump-app/` |
| 9 | SvelteKit scaffold + theme | `ui/` |
| 10 | Dashboard components | Sidebar, ProjectCard, etc. |
| 11 | Project detail page | Split panel, PhaseCard, etc. |
| 12 | Create/edit forms | NodeForm |
| 13 | End-to-end verification | Smoke tests |
