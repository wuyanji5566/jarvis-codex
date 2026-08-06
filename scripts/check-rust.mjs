import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const manifest = join(root, "src-tauri", "Cargo.toml");
const config = join(root, "src-tauri", "tauri.conf.json");
const env = { ...process.env, TAURI_CONFIG: readFileSync(config, "utf8") };

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

run("cargo", ["fmt", "--manifest-path", manifest, "--check"]);
run("cargo", ["clippy", "--manifest-path", manifest, "--all-targets", "--", "-D", "warnings"]);
