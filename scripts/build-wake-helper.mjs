import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));

if (process.platform === "darwin") {
  execFileSync("zsh", [join(root, "scripts", "build-wake-helper.sh")], {
    cwd: root,
    stdio: "inherit",
  });
} else if (process.platform === "win32") {
  const project = join(root, "src-tauri", "wake-helper", "windows", "JarvisWakeListener.csproj");
  if (existsSync(project)) {
    try {
      execFileSync("dotnet", ["--version"], { stdio: "ignore" });
    } catch {
      console.warn("dotnet is not installed; continuing with manual microphone mode.");
      process.exit(0);
    }
    execFileSync("dotnet", [
      "publish",
      project,
      "--configuration",
      "Release",
      "--runtime",
      "win-x64",
      "--self-contained",
      "true",
      "/p:PublishSingleFile=true",
      "/p:IncludeNativeLibrariesForSelfExtract=true",
      "--output",
      join(root, "src-tauri", "wake-helper", "windows", "publish"),
    ], { cwd: root, stdio: "inherit" });
  } else {
    console.log("Windows wake helper is not installed yet; continuing with manual microphone mode.");
  }
} else {
  console.log("Wake helper build is unsupported on " + process.platform + "; continuing without it.");
}
