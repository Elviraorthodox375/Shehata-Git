import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const explicitTargetIndex = process.argv.indexOf("--target");
const explicitTarget =
  explicitTargetIndex >= 0 ? process.argv[explicitTargetIndex + 1] : undefined;

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
    shell: false,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
}

function output(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
    shell: false,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(result.stderr || `${command} exited with status ${result.status}`);
  }
  return result.stdout;
}

const hostLine = output("rustc", ["-vV"])
  .split(/\r?\n/)
  .find((line) => line.startsWith("host: "));
if (!hostLine) throw new Error("rustc did not report a host target triple");

const target = explicitTarget || hostLine.slice("host: ".length).trim();
const cargoArgs = [
  "build",
  "--release",
  "-p",
  "shehata-cli",
  "-p",
  "shehata-credential-helper",
  "-p",
  "shehata-mcp",
];
if (explicitTarget) cargoArgs.push("--target", target);
run("cargo", cargoArgs);

const configuredTargetDir = process.env.CARGO_TARGET_DIR;
const targetRoot = configuredTargetDir
  ? isAbsolute(configuredTargetDir)
    ? configuredTargetDir
    : resolve(repoRoot, configuredTargetDir)
  : join(repoRoot, "target");
const releaseDir = explicitTarget
  ? join(targetRoot, target, "release")
  : join(targetRoot, "release");
const destinationDir = join(repoRoot, "apps", "desktop", "src-tauri", "binaries");
mkdirSync(destinationDir, { recursive: true });

const extension = target.includes("windows") ? ".exe" : "";
for (const name of ["shehata", "git-credential-shehata", "shehata-mcp"]) {
  const source = join(releaseDir, `${name}${extension}`);
  if (!existsSync(source)) throw new Error(`missing built sidecar: ${source}`);
  const destination = join(destinationDir, `${name}-${target}${extension}`);
  copyFileSync(source, destination);
  process.stdout.write(`Prepared ${destination}\n`);
}
