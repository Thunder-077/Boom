import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const rootDir = resolve(import.meta.dirname, "..");
const versionPattern = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;

const files = {
  packageJson: resolve(rootDir, "package.json"),
  tauriConfig: resolve(rootDir, "src-tauri", "tauri.conf.json"),
  cargoToml: resolve(rootDir, "src-tauri", "Cargo.toml"),
};

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function writeJson(path, data) {
  writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

function readCargoVersion() {
  const content = readFileSync(files.cargoToml, "utf8");
  const match = content.match(/^version\s*=\s*"([^"]+)"/m);
  if (!match) {
    throw new Error("未在 src-tauri/Cargo.toml 中找到 package.version");
  }
  return match[1];
}

function writeCargoVersion(version) {
  const content = readFileSync(files.cargoToml, "utf8");
  const next = content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  if (next === content) {
    throw new Error("未能更新 src-tauri/Cargo.toml 中的 package.version");
  }
  writeFileSync(files.cargoToml, next, "utf8");
}

function readVersions() {
  return {
    packageJson: readJson(files.packageJson).version,
    tauriConfig: readJson(files.tauriConfig).version,
    cargoToml: readCargoVersion(),
  };
}

function assertVersion(version) {
  if (!versionPattern.test(version)) {
    throw new Error(`版本号格式不正确: ${version}，请使用类似 0.1.1 的 semver 格式`);
  }
}

function checkVersions() {
  const versions = readVersions();
  const uniqueVersions = new Set(Object.values(versions));
  if (uniqueVersions.size !== 1) {
    console.error("版本号不一致:");
    for (const [name, version] of Object.entries(versions)) {
      console.error(`- ${name}: ${version}`);
    }
    process.exitCode = 1;
    return;
  }
  console.log(`版本号一致: ${versions.packageJson}`);
}

function setVersion(version) {
  assertVersion(version);

  const packageJson = readJson(files.packageJson);
  const tauriConfig = readJson(files.tauriConfig);

  packageJson.version = version;
  tauriConfig.version = version;

  writeJson(files.packageJson, packageJson);
  writeJson(files.tauriConfig, tauriConfig);
  writeCargoVersion(version);

  console.log(`已同步版本号为 ${version}`);
}

const [command, version] = process.argv.slice(2);

try {
  if (command === "check") {
    checkVersions();
  } else if (command === "set" && version) {
    setVersion(version);
    checkVersions();
  } else {
    console.error("用法: node scripts/sync-version.mjs check | set <version>");
    process.exitCode = 1;
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
