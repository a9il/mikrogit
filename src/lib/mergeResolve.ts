import type { ConflictFile } from "./types";

export function isResolved(choice: string | undefined): boolean {
  if (!choice) return false;
  return (
    choice === "current" ||
    choice === "incoming" ||
    choice === "both" ||
    choice.startsWith("edit:")
  );
}

/**
 * Applies per-region choices ("current" | "incoming" | "both" | "edit:<text>")
 * to the original conflicted content and returns the resolved file text.
 * Regions without a choice keep their conflict markers.
 */
export function resolvedContent(cf: ConflictFile, choices: string[]): string {
  const lines = cf.original_content.split("\n");
  if (lines.length > 1 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  let out = "";
  let regionIdx = 0;
  let i = 0;
  while (i < lines.length) {
    const trimmed = lines[i].trimStart();
    if (trimmed.startsWith("<<<<<<< ")) {
      const start = i;
      const currentLines: string[] = [];
      const incomingLines: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].trimStart().startsWith("=======")) {
        currentLines.push(lines[i]);
        i += 1;
      }
      if (i < lines.length) {
        i += 1;
      }
      while (i < lines.length && !lines[i].trimStart().startsWith(">>>>>>> ")) {
        incomingLines.push(lines[i]);
        i += 1;
      }
      const end = i;
      if (i < lines.length) {
        i += 1;
      }
      let choice = choices[regionIdx] ?? "";
      regionIdx += 1;
      let customLines: string | null = null;
      if (choice.startsWith("edit:")) {
        customLines = choice.slice(5);
        choice = "";
      }
      if (choice === "current") {
        for (const l of currentLines) out += l + "\n";
      } else if (choice === "incoming") {
        for (const l of incomingLines) out += l + "\n";
      } else if (choice === "both") {
        for (const l of currentLines) out += l + "\n";
        for (const l of incomingLines) out += l + "\n";
      } else if (customLines != null) {
        out += customLines;
        if (!customLines.endsWith("\n")) out += "\n";
      } else {
        for (let idx = start; idx <= end; idx++) out += lines[idx] + "\n";
      }
    } else {
      out += lines[i] + "\n";
      i += 1;
    }
  }
  return out;
}
