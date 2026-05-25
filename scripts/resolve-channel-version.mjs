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
const now = new Date();
const utcYear = now.getUTCFullYear() % 100;
const startOfYear = Date.UTC(now.getUTCFullYear(), 0, 1);
const currentDay = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
const dayOfYear = Math.floor((currentDay - startOfYear) / 86400000) + 1;
const buildDate = now.toISOString().slice(0, 10).replaceAll("-", "");

// MSI 要求 pre-release 标识只能是 <= 65535 的数字段，因此这里使用 yy.dayOfYear.runNumber。
// build metadata 继续携带 canary 与日期信息，便于界面展示和问题追踪。
const prerelease = `${utcYear}.${dayOfYear}.${runNumber}`;
const version = `${baseVersion}-${prerelease}+canary.${buildDate}.${shortSha}`;
const tag = `canary-v${baseVersion}-${prerelease}`;

console.log(
  JSON.stringify({
    channel,
    version,
    tag,
    shortSha,
  }),
);
