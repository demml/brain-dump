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
