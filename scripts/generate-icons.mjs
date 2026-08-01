/**
 * Generate every desktop icon from the canonical responsive SVG mark.
 * Tauri's icon command uses the same renderer as the application bundle and
 * produces Windows, macOS, Linux, Store, and mobile sizes consistently.
 */
import { spawnSync } from "node:child_process";
import { rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const desktop = join(root, "apps", "desktop");
const command = "pnpm exec tauri icon public/logo-mark.svg --output src-tauri/icons";
const result = spawnSync(command, { cwd: desktop, stdio: "inherit", shell: true });

if (result.error) throw result.error;
process.exitCode = result.status ?? 1;

if (result.status === 0) {
  // This product ships on desktop. Keep Windows and macOS outputs, while
  // removing mobile-only trees emitted by the general-purpose Tauri command.
  rmSync(join(desktop, "src-tauri", "icons", "android"), { recursive: true, force: true });
  rmSync(join(desktop, "src-tauri", "icons", "ios"), { recursive: true, force: true });
  for (const file of [
    "64x64.png",
    "StoreLogo.png",
    "Square30x30Logo.png",
    "Square44x44Logo.png",
    "Square71x71Logo.png",
    "Square89x89Logo.png",
    "Square107x107Logo.png",
    "Square142x142Logo.png",
    "Square150x150Logo.png",
    "Square284x284Logo.png",
    "Square310x310Logo.png",
  ]) {
    rmSync(join(desktop, "src-tauri", "icons", file), { force: true });
  }
}
