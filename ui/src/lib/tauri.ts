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
  status?: "active" | "completed" | "archived",
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
