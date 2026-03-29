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
    conn: &mut Connection,
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

    let sp = conn.savepoint()?;
    sp.execute(
        "INSERT INTO nodes (id, type, title, description, status, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'active', 0, ?5, ?5)",
        params![id, node_type.as_str(), title, description, now],
    )?;

    if let Some(pid) = parent_id {
        let edge_id = new_id();
        sp.execute(
            "INSERT INTO edges (id, source_id, target_id, edge_type, created_at)
             VALUES (?1, ?2, ?3, 'parent', ?4)",
            params![edge_id, pid, id, now],
        )?;
    }
    sp.commit()?;

    get_node(conn, &id)
}

pub fn get_node(conn: &Connection, id: &str) -> DbResult<Node> {
    conn.query_row(
        "SELECT id, type, title, description, status, sort_order, created_at, updated_at
         FROM nodes WHERE id = ?1",
        params![id],
        |row| {
            let node_type_str: String = row.get(1)?;
            let status_str: String = row.get(4)?;
            Ok(Node {
                id: row.get(0)?,
                node_type: node_type_str
                    .parse::<NodeType>()
                    .map_err(rusqlite::Error::InvalidColumnName)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: status_str
                    .parse::<NodeStatus>()
                    .map_err(rusqlite::Error::InvalidColumnName)?,
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
        let node_type_str: String = row.get(1)?;
        let status_str: String = row.get(4)?;
        Ok(Node {
            id: row.get(0)?,
            node_type: node_type_str
                .parse::<NodeType>()
                .map_err(rusqlite::Error::InvalidColumnName)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: status_str
                .parse::<NodeStatus>()
                .map_err(rusqlite::Error::InvalidColumnName)?,
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

pub fn get_children(conn: &Connection, parent_id: &str) -> DbResult<Vec<Node>> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.type, n.title, n.description, n.status, n.sort_order, n.created_at, n.updated_at
         FROM nodes n INNER JOIN edges e ON e.target_id = n.id
         WHERE e.source_id = ?1 AND e.edge_type = 'parent'
         ORDER BY n.sort_order, n.created_at"
    )?;
    let rows = stmt.query_map(params![parent_id], |row| {
        let node_type_str: String = row.get(1)?;
        let status_str: String = row.get(4)?;
        Ok(Node {
            id: row.get(0)?,
            node_type: node_type_str
                .parse::<NodeType>()
                .map_err(rusqlite::Error::InvalidColumnName)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: status_str
                .parse::<NodeStatus>()
                .map_err(rusqlite::Error::InvalidColumnName)?,
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::Sqlite)
}

pub fn delete_node(conn: &mut Connection, id: &str) -> DbResult<()> {
    let _ = get_node(conn, id)?;
    let sp = conn.savepoint()?;
    delete_node_recursive(&sp, id)?;
    sp.commit()?;
    Ok(())
}

fn delete_node_recursive(conn: &Connection, id: &str) -> DbResult<()> {
    let children = get_children(conn, id)?;
    for child in &children {
        delete_node_recursive(conn, &child.id)?;
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
        db::init_in_memory(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_project() {
        let mut conn = setup();
        let node = create_node(&mut conn, NodeType::Project, "Test Project", None).unwrap();
        assert_eq!(node.title, "Test Project");
        assert_eq!(node.node_type, NodeType::Project);
        assert_eq!(node.status, NodeStatus::Active);
        assert!(node.description.contains("## Goal"));

        let fetched = get_node(&conn, &node.id).unwrap();
        assert_eq!(fetched.id, node.id);
    }

    #[test]
    fn test_create_phase_under_project() {
        let mut conn = setup();
        let project = create_node(&mut conn, NodeType::Project, "P1", None).unwrap();
        let phase = create_node(&mut conn, NodeType::Phase, "Phase 1", Some(&project.id)).unwrap();
        assert_eq!(phase.node_type, NodeType::Phase);

        let phases = list_nodes(&conn, NodeType::Phase, Some(&project.id), None, None).unwrap();
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].id, phase.id);
    }

    #[test]
    fn test_create_task_under_phase() {
        let mut conn = setup();
        let project = create_node(&mut conn, NodeType::Project, "P1", None).unwrap();
        let phase = create_node(&mut conn, NodeType::Phase, "Phase 1", Some(&project.id)).unwrap();
        let task = create_node(&mut conn, NodeType::Task, "Task 1", Some(&phase.id)).unwrap();
        assert_eq!(task.node_type, NodeType::Task);
    }

    #[test]
    fn test_hierarchy_enforcement() {
        let mut conn = setup();
        let project = create_node(&mut conn, NodeType::Project, "P1", None).unwrap();
        // Can't add a task directly under a project
        let err = create_node(&mut conn, NodeType::Task, "T1", Some(&project.id));
        assert!(err.is_err());
    }

    #[test]
    fn test_update_node_status_logs_history() {
        let mut conn = setup();
        let node = create_node(&mut conn, NodeType::Project, "P1", None).unwrap();
        update_node(&conn, &node.id, None, None, Some(NodeStatus::Completed), None).unwrap();

        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM status_history WHERE node_id = ?1",
            params![node.id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_delete_node_cascades_children() {
        let mut conn = setup();
        let project = create_node(&mut conn, NodeType::Project, "P1", None).unwrap();
        let phase = create_node(&mut conn, NodeType::Phase, "Phase 1", Some(&project.id)).unwrap();
        let task = create_node(&mut conn, NodeType::Task, "Task 1", Some(&phase.id)).unwrap();

        delete_node(&mut conn, &project.id).unwrap();

        // Phase should also be gone (recursive delete)
        assert!(get_node(&conn, &phase.id).is_err());
        // Also verify the task (grandchild) is gone
        assert!(get_node(&conn, &task.id).is_err());
    }

    #[test]
    fn test_list_projects_by_status() {
        let mut conn = setup();
        create_node(&mut conn, NodeType::Project, "Active", None).unwrap();
        let p2 = create_node(&mut conn, NodeType::Project, "Done", None).unwrap();
        update_node(&conn, &p2.id, None, None, Some(NodeStatus::Completed), None).unwrap();

        let active = list_nodes(&conn, NodeType::Project, None, Some(NodeStatus::Active), None).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "Active");
    }
}
