import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const manifest = join(root, "src-tauri", "Cargo.toml");
const baseConfigPath = join(root, "src-tauri", "tauri.conf.json");
const platformConfigPath = join(root, "src-tauri", "tauri.windows.conf.json");
const baseConfig = JSON.parse(readFileSync(baseConfigPath, "utf8"));
const platformConfig = process.platform === "win32"
  ? JSON.parse(readFileSync(platformConfigPath, "utf8"))
  : {};
const mergedConfig = {
  ...baseConfig,
  ...platformConfig,
  app: { ...baseConfig.app, ...platformConfig.app },
  build: { ...baseConfig.build, ...platformConfig.build },
  bundle: { ...baseConfig.bundle, ...platformConfig.bundle },
};
const env = { ...process.env, TAURI_CONFIG: JSON.stringify(mergedConfig) };

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

run("cargo", ["fmt", "--manifest-path", manifest, "--check"]);
run("cargo", ["clippy", "--manifest-path", manifest, "--all-targets", "--", "-D", "warnings"]);
