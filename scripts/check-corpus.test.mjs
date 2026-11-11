import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const script = fileURLToPath(new URL("./check-corpus.sh", import.meta.url));

test("corpus gate preserves filenames and refuses incomplete discovery", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "m1-corpus-gate-"));
  const hidden = path.join(root, "unreadable");
  const run = () => spawnSync("bash", [script, root], { encoding: "utf8", timeout: 30000 });
  try {
    fs.writeFileSync(path.join(root, "normal.m1scr"), "local value = 1;\n");
    fs.writeFileSync(path.join(root, "line\nbreak.m1scr"), "local value = 2;\n");
    fs.mkdirSync(path.join(root, "directory.m1scr"));
    const valid = run();
    assert.equal(valid.status, 0, valid.stdout + valid.stderr);
    assert.match(valid.stdout, /parsed 2 scripts/);

    fs.mkdirSync(hidden);
    fs.writeFileSync(path.join(hidden, "invalid.m1scr"), "local value = ;\n");
    const invalid = run();
    assert.notEqual(invalid.status, 0, "malformed source must fail");

    // Root can read mode-000 directories, so use the normal CI/user account.
    if (process.getuid?.() !== 0) {
      fs.chmodSync(hidden, 0);
      const inaccessible = run();
      assert.notEqual(inaccessible.status, 0, "partial discovery must fail");
      assert.match(inaccessible.stderr, /discover|Permission denied/);
      assert.doesNotMatch(inaccessible.stdout, /OK.*corpus parses clean/);
    }
  } finally {
    if (fs.existsSync(hidden)) fs.chmodSync(hidden, 0o700);
    fs.rmSync(root, { recursive: true, force: true });
  }
});
