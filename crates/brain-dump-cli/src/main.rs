use std::process;

use brain_dump_core::db;
use brain_dump_core::models::*;
use brain_dump_core::queries;
use clap::{Parser, Subcommand};
use rusqlite::Connection;

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
    let db_path = match db::default_db_path() {
        Ok(p) => p,
        Err(e) => {
            err_exit(&cli, &format!("cannot determine database path: {e}"));
        }
    };

    let mut conn = match db::open_db(&db_path) {
        Ok(c) => c,
        Err(e) => {
            err_exit(&cli, &format!("failed to open database: {e}"));
        }
    };

    if let Err(e) = run(&cli, &mut conn) {
        err_exit(&cli, &e.to_string());
    }
}

fn run(cli: &Cli, conn: &mut Connection) -> Result<(), brain_dump_core::db::DbError> {
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
                    let s = status
                        .as_deref()
                        .map(|s| s.parse::<NodeStatus>().map_err(brain_dump_core::db::DbError::Custom))
                        .transpose()?;
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
                println!(
                    "{}",
                    serde_json::to_string_pretty(&detail).map_err(|e| brain_dump_core::db::DbError::Custom(e.to_string()))?
                );
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
            let s = status
                .parse::<NodeStatus>()
                .map_err(brain_dump_core::db::DbError::Custom)?;
            let node = queries::update_node(conn, &resolved, None, None, Some(s), None)?;
            output_node(cli, &node);
        }

        Commands::Link { source, edge_type, target } => {
            let src = queries::resolve_node(conn, source)?;
            let tgt = queries::resolve_node(conn, target)?;
            let et = edge_type
                .parse::<EdgeType>()
                .map_err(brain_dump_core::db::DbError::Custom)?;
            let edge = queries::create_edge(conn, &src, &tgt, et)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&edge).map_err(|e| brain_dump_core::db::DbError::Custom(e.to_string()))?
                );
            } else {
                println!("Linked {} --[{}]--> {}", source, edge_type, target);
            }
        }

        Commands::Unlink { source, edge_type, target } => {
            let src = queries::resolve_node(conn, source)?;
            let tgt = queries::resolve_node(conn, target)?;
            let et = edge_type
                .parse::<EdgeType>()
                .map_err(brain_dump_core::db::DbError::Custom)?;
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
        println!("{}", serde_json::to_string_pretty(node).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}") ));
    } else {
        println!(
            "[{}] {} — {} ({})",
            node.node_type.as_str(),
            node.id,
            node.title,
            node.status.as_str()
        );
    }
}

fn output_nodes(cli: &Cli, nodes: &[Node]) {
    if cli.json {
        println!("{}", serde_json::to_string_pretty(nodes).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}") ));
    } else {
        if nodes.is_empty() {
            println!("No results.");
            return;
        }
        for node in nodes {
            println!(
                "[{}] {} — {} ({})",
                node.node_type.as_str(),
                node.id,
                node.title,
                node.status.as_str()
            );
        }
    }
}

fn print_detail(detail: &NodeDetail) {
    let n = &detail.node;
    println!("=== {} ===", n.title);
    println!(
        "Type: {}  Status: {}  ID: {}",
        n.node_type.as_str(),
        n.status.as_str(),
        n.id
    );

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
            let mark = if child.status == NodeStatus::Completed { "✓" } else { "○" };
            println!(
                "  {mark} [{}] {} — {}",
                child.node_type.as_str(),
                child.id,
                child.title
            );
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
    std::fs::write(&tmp, content).map_err(brain_dump_core::db::DbError::Io)?;

    let status = std::process::Command::new(&editor)
        .arg(&tmp)
        .status()
        .map_err(brain_dump_core::db::DbError::Io)?;

    if !status.success() {
        return Err(brain_dump_core::db::DbError::Custom(
            "editor exited with error".to_string(),
        ));
    }

    std::fs::read_to_string(&tmp).map_err(brain_dump_core::db::DbError::Io)
}

fn err_exit(cli: &Cli, msg: &str) -> ! {
    if cli.json {
        eprintln!("{}", serde_json::json!({"error": msg}));
    } else {
        eprintln!("error: {msg}");
    }
    process::exit(1);
}
