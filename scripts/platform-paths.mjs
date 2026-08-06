import path from "node:path";

function pushUnique(result, value) {
  if (typeof value !== "string" || value.length === 0 || result.includes(value)) return;
  result.push(value);
}

export function buildCodexCandidates({ platform = process.platform, env = process.env, where = [] } = {}) {
  const result = [];
  pushUnique(result, env.JARVIS_CODEX_BIN);

  if (platform === "win32") {
    for (const value of where) pushUnique(result, value);
    if (env.USERPROFILE) {
      pushUnique(result, path.win32.join(env.USERPROFILE, "bin", "codex.exe"));
      pushUnique(result, path.win32.join(env.USERPROFILE, ".codex", "bin", "codex.exe"));
    }
    if (env.LOCALAPPDATA) {
      pushUnique(result, path.win32.join(env.LOCALAPPDATA, "OpenAI", "Codex", "bin", "codex.exe"));
    }
    if (env.APPDATA) {
      pushUnique(result, path.win32.join(env.APPDATA, "npm", "codex.cmd"));
      pushUnique(result, path.win32.join(env.APPDATA, "npm", "codex.exe"));
    }
    return result;
  }

  for (const value of where) pushUnique(result, value);
  if (platform === "darwin") {
    pushUnique(result, "/Applications/ChatGPT.app/Contents/Resources/codex");
    pushUnique(result, "/Applications/Codex.app/Contents/Resources/codex");
    pushUnique(result, "/opt/homebrew/bin/codex");
    pushUnique(result, "/usr/local/bin/codex");
    if (env.HOME) {
      pushUnique(result, path.posix.join(env.HOME, ".local/bin/codex"));
      pushUnique(result, path.posix.join(env.HOME, ".cargo/bin/codex"));
    }
  }
  return result;
}

export function normalizeWorkspacePath(value) {
  const trimmed = String(value ?? "").trim();
  if (trimmed.startsWith("\\\\?\\UNC\\")) return "\\\\" + trimmed.slice(8);
  if (trimmed.startsWith("\\\\?\\")) return trimmed.slice(4);
  return trimmed;
}
