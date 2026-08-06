import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const frontend = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
const wakeHelper = await readFile(
  new URL("../src-tauri/wake-helper/windows/Program.cs", import.meta.url),
  "utf8",
);

test("杰克品牌和唤醒词贯穿 Windows 前端与本地监听器", () => {
  assert.match(frontend, /杰克/);
  assert.match(frontend, /嘿，杰克/);
  assert.match(wakeHelper, /嘿，杰克/);
  assert.match(frontend, /jarvis-character-v2\.png/);
});

test("Voice 使用 V3 支持的科技感语音 cove", () => {
  assert.match(frontend, /voice:\s*["']cove["']/);
});
