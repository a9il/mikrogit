import type { FileEntry } from "./types";

export interface TreeNode {
  name: string;
  path: string;
  dir: boolean;
  depth: number;
  children: TreeNode[];
  files: FileEntry[];
}

export function buildFileTree(files: FileEntry[]): TreeNode[] {
  const root: TreeNode = {
    name: "",
    path: "",
    dir: true,
    depth: -1,
    children: [],
    files: [],
  };

  for (const f of files) {
    const parts = f.path.split("/");
    let node = root;
    let prefix = "";
    for (let i = 0; i < parts.length - 1; i++) {
      prefix = prefix ? `${prefix}/${parts[i]}` : parts[i];
      let child = node.children.find((c) => c.name === parts[i]);
      if (!child) {
        child = {
          name: parts[i],
          path: prefix,
          dir: true,
          depth: node.depth + 1,
          children: [],
          files: [],
        };
        node.children.push(child);
        node.children.sort((a, b) => a.name.localeCompare(b.name));
      }
      node = child;
    }
    node.files.push(f);
  }

  const sortRec = (n: TreeNode) => {
    n.files.sort((a, b) => a.path.localeCompare(b.path));
    for (const c of n.children) sortRec(c);
  };
  sortRec(root);
  return root.children.length > 0 || root.files.length > 0 ? [root] : [];
}

export function flattenVisible(
  nodes: TreeNode[],
  collapsed: Set<string>,
): Array<{ node: TreeNode; file?: FileEntry }> {
  const out: Array<{ node: TreeNode; file?: FileEntry }> = [];
  const walk = (n: TreeNode) => {
    if (n.depth >= 0) {
      out.push({ node: n });
      if (collapsed.has(n.path)) return;
    }
    for (const c of n.children) walk(c);
    for (const f of n.files) out.push({ node: n, file: f });
  };
  for (const n of nodes) walk(n);
  return out;
}

export function isFlatList(files: FileEntry[]): boolean {
  return !files.some((f) => f.path.includes("/"));
}
