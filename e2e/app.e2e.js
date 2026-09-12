// Drives the real mikrogit window through WebDriver (tauri-driver).
// Regression coverage: merge editor, stage/commit, commit-message retention,
// group collapse, stage-all, one-click folder expand, explorer edit+save,
// terminal commands.
import { remote } from "webdriverio";

const APP = process.env.E2E_APP;
const PORT = Number(process.env.E2E_PORT || 4444);
if (!APP) throw new Error("E2E_APP not set");

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let client;
try {
  client = await remote({
    hostname: "127.0.0.1",
    port: PORT,
    capabilities: { "tauri:options": { application: APP } },
    logLevel: "error",
    waitforTimeout: 15000,
  });

  const step = async (name, fn) => {
    process.stdout.write(`- ${name} … `);
    await fn();
    console.log("ok");
  };

  await step("app opened the fixture repo", async () => {
    const el = await client.$('[data-testid="repo-path"]');
    await el.waitForExist();
    let txt = "";
    for (let i = 0; i < 40; i++) {
      // getText() drops text clipped by overflow:hidden; title is stable.
      txt = (await el.getAttribute("title")) || "";
      if (txt.includes("fixture-repo")) break;
      await sleep(250);
    }
    if (!txt.includes("fixture-repo")) throw new Error(`repo path wrong: ${JSON.stringify(txt)}`);
  });

  await step("status bar shows branch main with pending changes", async () => {
    const counts = await client.$('[data-testid="status-counts"]');
    await counts.waitForExist();
    const txt = await counts.getText();
    if (!/modified|untracked/.test(txt)) throw new Error(`expected pending changes, got: ${txt}`);
  });

  await step("merge editor resolves the conflict", async () => {
    const row = await client.$('.file-row[title="file.txt"]');
    await row.waitForExist();
    await row.click();
    const incoming = await client.$("button=Incoming");
    await incoming.waitForExist();
    await incoming.click();
    const save = await client.$("button*=Save");
    await save.waitForEnabled();
    await save.click();
    await sleep(800);
  });

  await step("stage the modified file from the changes tree", async () => {
    // Row actions are hover-revealed; move the pointer onto the row first.
    const row = await client.$('.file-row[title="README.md"]');
    await row.waitForExist();
    await row.moveTo();
    await sleep(300);
    const btn = await row.$('[title="Stage"]');
    await btn.waitForExist();
    await btn.click();
    await sleep(600);
    const counts = await client.$('[data-testid="status-counts"]');
    const txt = await counts.getText();
    if (!txt.includes("staged 2")) throw new Error(`file not staged, got: ${txt}`);
  });

  await step("commit staged changes", async () => {
    const msg = await client.$('[data-testid="commit-message"]');
    await msg.waitForExist();
    await msg.setValue("e2e commit");
    const btn = await client.$('[data-testid="commit-button"]');
    await btn.waitForEnabled();
    await btn.click();
    await sleep(800);
    const counts = await client.$('[data-testid="status-counts"]');
    const txt = await counts.getText();
    if (!txt.includes("untracked 1")) throw new Error(`commit not reflected, got: ${txt}`);
  });

  await step("commit message persists across view switches", async () => {
    const msg = await client.$('[data-testid="commit-message"]');
    await msg.setValue("retention check");
    await client.$('[title="Explorer"]').click();
    await client.$('[title="Source Control"]').click();
    const again = await client.$('[data-testid="commit-message"]');
    await again.waitForExist();
    const val = await again.getValue();
    if (val !== "retention check") throw new Error(`message lost, got: ${JSON.stringify(val)}`);
    await again.clearValue();
  });

  await step("group collapse hides rows; second click shows them", async () => {
    const header = await client.$(
      '//div[contains(@class,"group-header") and contains(.,"Untracked")]',
    );
    await header.waitForExist();
    await header.click();
    const row = await client.$('.file-row[title="notes.txt"]');
    await row.waitForExist({ reverse: true, timeout: 3000 });
    await header.click();
    await row.waitForExist({ timeout: 5000 });
  });

  await step("stage-all button on group header stages the group", async () => {
    const header = await client.$(
      '//div[contains(@class,"group-header") and contains(.,"Untracked")]',
    );
    await header.waitForExist();
    await header.moveTo();
    await sleep(300);
    const btn = await header.$('[title="Stage all"]');
    await btn.waitForExist();
    await btn.click();
    await sleep(600);
    const counts = await client.$('[data-testid="status-counts"]');
    const txt = await counts.getText();
    if (!txt.includes("staged 1")) throw new Error(`stage-all not reflected, got: ${txt}`);
  });

  await step("explorer lists files and previews content", async () => {
    await client.$('[title="Explorer"]').click();
    const row = await client.$('.file-row[title="notes.txt"]');
    await row.waitForExist();
    await row.click();
    const preview = await client.$("main textarea");
    await preview.waitForExist();
    const content = await preview.getValue();
    if (!content.includes("untracked file")) throw new Error(`bad preview: ${JSON.stringify(content)}`);
  });

  await step("folder expands with one click", async () => {
    const folder = await client.$('.folder-row[title="src"]');
    await folder.waitForExist();
    const child = await client.$('.file-row[title="src/app.js"]');
    await child.waitForExist({ reverse: true, timeout: 2000 });
    await folder.click();
    await child.waitForExist({ timeout: 5000 });
  });

  await step("edit file in explorer and save", async () => {
    const row = await client.$('.file-row[title="notes.txt"]');
    await row.click();
    const ta = await client.$("main textarea");
    await ta.waitForExist();
    await ta.setValue("edited by e2e\n");
    const save = await client.$('[data-testid="explorer-save"]');
    await save.waitForEnabled();
    await save.click();
    await sleep(600);
    if (await save.isEnabled()) throw new Error("save button still enabled after save");
    // Round trip: open another file, come back, edited content must be on disk.
    await (await client.$('.file-row[title="README.md"]')).click();
    await (await client.$('.file-row[title="notes.txt"]')).click();
    const ta2 = await client.$("main textarea");
    await ta2.waitForExist();
    const val = await ta2.getValue();
    // WebKit strips the trailing newline typed into the textarea; content from
    // disk must still round-trip.
    if (val.trimEnd() !== "edited by e2e") throw new Error(`save round trip failed: ${JSON.stringify(val)}`);
  });

  await step("changes view still works after explorer use", async () => {
    await client.$('[title="Source Control"]').click();
    const btn = await client.$('[data-testid="commit-button"]');
    await btn.waitForExist();
  });

  await step("terminal runs a git command and shows output", async () => {
    await client.$('[title="Terminal"]').click();
    const input = await client.$('[data-testid="terminal-input"]');
    await input.waitForExist();
    await input.setValue("log --oneline -5");
    await client.keys(["Enter"]);
    await sleep(800);
    const out = await client.$('[data-testid="terminal-output"]');
    const txt = await out.getText();
    if (!txt.includes("e2e commit")) throw new Error(`log output missing commit: ${txt}`);
  });

  await client.deleteSession();
} catch (err) {
  console.error("\nE2E FAILED:", err && err.message ? err.message : err);
  try {
    if (client) await client.deleteSession();
  } catch {}
  process.exit(1);
}
