export function parseUnifiedDiff(
  text: string,
  path: string,
): import("./types").FileDiff {
  const hunks: import("./types").FileDiff["hunks"] = [];
  let added = 0;
  let removed = 0;
  let binary = false;
  let old_path: string | undefined;

  const lines = text.split("\n");
  let current: import("./types").FileDiff["hunks"][number] | null = null;
  let old_no = 0;
  let new_no = 0;

  for (const raw of lines) {
    if (
      raw.startsWith("Binary files ") ||
      raw.startsWith("GIT binary patch")
    ) {
      binary = true;
      continue;
    }
    if (raw.startsWith("--- ") && !raw.startsWith("--- /dev/null")) {
      old_path = raw.slice(4).replace(/^a\//, "").trim();
      continue;
    }
    if (raw.startsWith("+++ ")) continue;
    if (raw.startsWith("@@ ")) {
      const m = /@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/.exec(raw);
      if (!m) continue;
      current = {
        header: raw,
        old_start: Number(m[1]),
        old_lines: m[2] === undefined ? 1 : Number(m[2]),
        new_start: Number(m[3]),
        new_lines: m[4] === undefined ? 1 : Number(m[4]),
        lines: [],
      };
      hunks.push(current);
      old_no = current.old_start;
      new_no = current.new_start;
      continue;
    }
    if (!current) continue;
    if (raw.startsWith("+") && !raw.startsWith("+++")) {
      current.lines.push({ kind: "add", content: raw.slice(1), new_no });
      new_no++;
      added++;
    } else if (raw.startsWith("-") && !raw.startsWith("---")) {
      current.lines.push({ kind: "del", content: raw.slice(1), old_no });
      old_no++;
      removed++;
    } else if (raw.startsWith(" ")) {
      current.lines.push({
        kind: "context",
        content: raw.slice(1),
        old_no,
        new_no,
      });
      old_no++;
      new_no++;
    } else if (raw === "" || raw === "\\ No newline at end of file") {
      continue;
    } else if (raw.startsWith("\\")) {
      continue;
    }
  }

  return { path, old_path, hunks, binary, added, removed };
}
