import { describe, expect, it } from "vitest";
import { buildFileTree, flattenVisible, isFlatList } from "./fileTree";
import type { FileEntry } from "./types";

const f = (path: string): FileEntry => ({
  path,
  index_status: "modified",
  worktree_status: "modified",
});

describe("buildFileTree", () => {
  it("groups nested files under folders", () => {
    const tree = buildFileTree([f("src/a.ts"), f("src/b.ts"), f("root.ts")]);
    expect(tree).toHaveLength(1);
    const root = tree[0];
    expect(root.children.map((c) => c.name)).toEqual(["src"]);
    expect(root.files.map((x) => x.path)).toEqual(["root.ts"]);
    expect(root.children[0].files.map((x) => x.path)).toEqual([
      "src/a.ts",
      "src/b.ts",
    ]);
  });

  it("builds deep nesting", () => {
    const tree = buildFileTree([f("a/b/c.ts")]);
    const a = tree[0].children[0];
    expect(a.name).toBe("a");
    expect(a.children[0].name).toBe("b");
    expect(a.children[0].files[0].path).toBe("a/b/c.ts");
  });

  it("returns empty for no files", () => {
    expect(buildFileTree([])).toEqual([]);
  });
});

describe("flattenVisible", () => {
  it("hides children of collapsed folders", () => {
    const tree = buildFileTree([f("src/a.ts"), f("root.ts")]);
    const all = flattenVisible(tree, new Set());
    expect(all.filter((r) => r.file).map((r) => r.file!.path)).toEqual([
      "src/a.ts",
      "root.ts",
    ]);
    const collapsed = flattenVisible(tree, new Set(["src"]));
    expect(collapsed.filter((r) => r.file).map((r) => r.file!.path)).toEqual([
      "root.ts",
    ]);
  });
});

describe("isFlatList", () => {
  it("detects flat vs nested", () => {
    expect(isFlatList([f("a.ts")])).toBe(true);
    expect(isFlatList([f("a/b.ts")])).toBe(false);
  });
});
