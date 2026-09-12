import { describe, expect, it } from "vitest";
import { isResolved, resolvedContent } from "./mergeResolve";
import type { ConflictFile } from "./types";

const cf = (content: string): ConflictFile => ({
  path: "f.txt",
  regions: [],
  has_conflicts: true,
  original_content: content,
});

const SINGLE = "a\n<<<<<<< HEAD\ncur\n=======\ninc\n>>>>>>> feat\nc\n";

describe("isResolved", () => {
  it("treats only known choices as resolved", () => {
    expect(isResolved(undefined)).toBe(false);
    expect(isResolved("")).toBe(false);
    expect(isResolved("current")).toBe(true);
    expect(isResolved("incoming")).toBe(true);
    expect(isResolved("both")).toBe(true);
    expect(isResolved("edit:custom text")).toBe(true);
    expect(isResolved("bogus")).toBe(false);
  });
});

describe("resolvedContent", () => {
  it("accepts current", () => {
    expect(resolvedContent(cf(SINGLE), ["current"])).toBe("a\ncur\nc\n");
  });

  it("accepts incoming", () => {
    expect(resolvedContent(cf(SINGLE), ["incoming"])).toBe("a\ninc\nc\n");
  });

  it("accepts both in order", () => {
    expect(resolvedContent(cf(SINGLE), ["both"])).toBe("a\ncur\ninc\nc\n");
  });

  it("keeps markers when a region is unresolved", () => {
    expect(resolvedContent(cf(SINGLE), [])).toBe(SINGLE);
  });

  it("applies choices per region", () => {
    const two =
      "<<<<<<< HEAD\n1c\n=======\n1i\n>>>>>>> a\nmid\n<<<<<<< HEAD\n2c\n=======\n2i\n>>>>>>> b\n";
    expect(resolvedContent(cf(two), ["current", "incoming"])).toBe("1c\nmid\n2i\n");
  });

  it("applies custom edited text", () => {
    expect(resolvedContent(cf(SINGLE), ["edit:merged line"])).toBe("a\nmerged line\nc\n");
    expect(resolvedContent(cf(SINGLE), ["edit:l1\nl2\n"])).toBe("a\nl1\nl2\nc\n");
  });

  it("preserves trailing newlines without doubling", () => {
    const content = "<<<<<<< HEAD\nx\n=======\ny\n>>>>>>> b\n";
    expect(resolvedContent(cf(content), ["current"])).toBe("x\n");
    expect(resolvedContent(cf("plain\n"), [])).toBe("plain\n");
    expect(resolvedContent(cf("plain"), [])).toBe("plain\n");
  });
});
