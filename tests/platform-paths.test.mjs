import test from "node:test";
import { readFile } from "node:fs/promises";

const windowsPlatform = await readFile(
  new URL("../src-tauri/src/platform/windows.rs", import.meta.url),
  "utf8",
);
import assert from "node:assert/strict";
import { buildCodexCandidates, normalizeWorkspacePath } from "../scripts/platform-paths.mjs";

test("Windows candidates prefer explicit binary and preserve Unicode paths", () => {
  const candidates = buildCodexCandidates({
    platform: "win32",
    env: {
      JARVIS_CODEX_BIN: "C:\\Program Files\\Codex\\codex.exe",
      LOCALAPPDATA: "C:\\Users\\张三\\AppData\\Local",
      USERPROFILE: "C:\\Users\\张三",
      APPDATA: "C:\\Users\\张三\\AppData\\Roaming",
    },
    where: ["C:\\Tools\\codex.exe", "C:\\Tools\\codex.cmd"],
  });

  assert.deepEqual(candidates.slice(0, 4), [
    "C:\\Program Files\\Codex\\codex.exe",
    "C:\\Tools\\codex.exe",
    "C:\\Tools\\codex.cmd",
    "C:\\Users\\张三\\bin\\codex.exe",
  ]);
  assert.equal(
    candidates[4],
    "C:\\Users\\张三\\.codex\\bin\\codex.exe",
  );
  assert.equal(
    candidates[5],
    "C:\\Users\\张三\\AppData\\Local\\OpenAI\\Codex\\bin\\codex.exe",
  );
});

test("Windows workspace paths are not converted to WSL paths", () => {
  const path = "C:\\Users\\张三\\项目 文件夹";
  assert.equal(normalizeWorkspacePath(path), path);
});

test("Windows Codex child inherits the configured system proxy", () => {
  assert.match(windowsPlatform, /ProxyEnable/);
  assert.match(windowsPlatform, /HTTPS_PROXY/);
});

test("Windows Codex child explicitly enables browser and desktop tool hosts", () => {
  assert.match(windowsPlatform, /code_mode_host/);
  assert.match(windowsPlatform, /browser_use/);
  assert.match(windowsPlatform, /computer_use/);
});
