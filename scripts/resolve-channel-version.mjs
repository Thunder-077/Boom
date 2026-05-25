import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const [channel] = process.argv.slice(2);

if (channel !== "canary") {
  throw new Error(`unsupported channel: ${channel ?? "<empty>"}`);
}

const rootDir = resolve(import.meta.dirname, "..");
const packageJson = JSON.parse(readFileSync(resolve(rootDir, "package.json"), "utf8"));
const baseVersion = String(packageJson.version).split("-")[0];
const runNumber = process.env.GITHUB_RUN_NUMBER ?? "0";
const shortSha = (process.env.GITHUB_SHA ?? "dev").slice(0, 7);
const buildDate = new Date().toISOString().slice(0, 10).replaceAll("-", "");

// Canary 版本保持唯一且单调递增，便于 GitHub Release 与 Tauri updater 做精确比较。
const version = `${baseVersion}-canary.${buildDate}.${runNumber}`;
const tag = `canary-v${version}`;

console.log(
  JSON.stringify({
    channel,
    version,
    tag,
    shortSha,
  }),
);
