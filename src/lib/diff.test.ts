import { describe, expect, it } from "vitest";
import { parseUnifiedDiff } from "./diff";

const SAMPLE = `diff --git a/src/app.ts b/src/app.ts
index 1111111..2222222 100644
--- a/src/app.ts
+++ b/src/app.ts
@@ -1,3 +1,4 @@
 line1
-line2
+line2 changed
+line3 added
 line4
`;

describe("parseUnifiedDiff", () => {
  it("parses hunks with line numbers", () => {
    const d = parseUnifiedDiff(SAMPLE, "src/app.ts");
    expect(d.binary).toBe(false);
    expect(d.hunks).toHaveLength(1);
    const h = d.hunks[0];
    expect(h.old_start).toBe(1);
    expect(h.new_start).toBe(1);
    expect(h.lines.map((l) => l.kind)).toEqual([
      "context",
      "del",
      "add",
      "add",
      "context",
    ]);
    expect(d.added).toBe(2);
    expect(d.removed).toBe(1);
    expect(h.lines[1].old_no).toBe(2);
    expect(h.lines[2].new_no).toBe(2);
  });

  it("detects binary diffs", () => {
    const d = parseUnifiedDiff(
      "diff --git a/a.png b/a.png\nBinary files a/a.png and b/a.png differ\n",
      "a.png",
    );
    expect(d.binary).toBe(true);
    expect(d.hunks).toHaveLength(0);
  });

  it("handles empty diff", () => {
    const d = parseUnifiedDiff("", "x.ts");
    expect(d.hunks).toHaveLength(0);
    expect(d.added).toBe(0);
  });
});
