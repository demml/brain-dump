use brain_dump_core::models::*;
use brain_dump_core::queries;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[tauri::command]
pub fn create_node(
    state: State<'_, AppState>,
    node_type: String,
    title: String,
    parent_id: Option<String>,
) -> Result<Node, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let conn = &mut *conn;
    let nt = node_type.parse::<NodeType>().map_err(|e| e)?;
    queries::create_node(conn, nt, &title, parent_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_node(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    sort_order: Option<i32>,
) -> Result<Node, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let s = match status.as_deref() {
        Some(s) => Some(s.parse::<NodeStatus>().map_err(|e| e)?),
        None => None,
    };
    queries::update_node(
        &*conn,
        &id,
        title.as_deref(),
        description.as_deref(),
        s,
        sort_order,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_node(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let conn = &mut *conn;
    queries::delete_node(conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_node(state: State<'_, AppState>, id: String) -> Result<NodeDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_node_detail(&*conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_nodes(
    state: State<'_, AppState>,
    node_type: String,
    parent_id: Option<String>,
    status: Option<String>,
    tag: Option<String>,
) -> Result<Vec<Node>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let nt = node_type.parse::<NodeType>().map_err(|e| e)?;
    let s = match status.as_deref() {
        Some(s) => Some(s.parse::<NodeStatus>().map_err(|e| e)?),
        None => None,
    };
    queries::list_nodes(&*conn, nt, parent_id.as_deref(), s, tag.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_edge(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
    edge_type: String,
) -> Result<Edge, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let conn = &mut *conn;
    let et = edge_type.parse::<EdgeType>().map_err(|e| e)?;
    queries::create_edge(conn, &source_id, &target_id, et).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_edge(
    state: State<'_, AppState>,
    source_id: String,
    edge_type: String,
    target_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let et = edge_type.parse::<EdgeType>().map_err(|e| e)?;
    queries::delete_edge(&*conn, &source_id, et, &target_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag(
    state: State<'_, AppState>,
    node_id: String,
    tag_name: String,
) -> Result<Tag, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_tag(&*conn, &node_id, &tag_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag(
    state: State<'_, AppState>,
    node_id: String,
    tag_name: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::remove_tag(&*conn, &node_id, &tag_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_tags(&*conn).map_err(|e| e.to_string())
}
