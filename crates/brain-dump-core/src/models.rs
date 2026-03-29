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
