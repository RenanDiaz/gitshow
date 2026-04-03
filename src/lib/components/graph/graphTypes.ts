export interface Ref {
  name: string;
  type: 'local' | 'remote' | 'tag' | 'head';
  upstream?: string;
  trackingStatus?: string;
}

export interface Commit {
  hash: string;
  parents: string[];
  author: string;
  email: string;
  timestamp: number;
  refs: Ref[];
  subject: string;
  column: number;
  row: number;
}

export interface GraphLane {
  color: string;
  commits: string[];
}

export interface GraphEdge {
  from: string;
  to: string;
  fromColumn: number;
  toColumn: number;
  fromRow: number;
  toRow: number;
  type: 'straight' | 'merge' | 'fork';
  color: string;
}

export const LANE_WIDTH = 16;
export const ROW_HEIGHT = 32;
export const NODE_RADIUS = 5;
export const GRAPH_PADDING = 20;
