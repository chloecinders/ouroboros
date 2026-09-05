import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const DEBUG = process.argv.slice(2).includes("--debug");

function step(name: string): Promise<void> {
    return new Promise((resolve, reject) => {
        const running = spawn(process.execPath, ["--disable-warning=ExperimentalWarning", path.join(path.dirname(fileURLToPath(import.meta.url)), name)], {
            stdio: "inherit",
            env: DEBUG ? { ...process.env, AEGIS_DEBUG_BUILD: "1" } : process.env,
        });

        running.on("error", reject);
        running.on("exit", (code, signal) => {
            if (signal) return reject(new Error(`${name} was killed by ${signal}`));
            if (code) return reject(new Error(`${name} exited with ${code}`));

            resolve();
        });
    });
}

for (const name of ["assets.ts", "site.ts", "minify.ts"]) await step(name);

console.log(DEBUG ? "Built the assets and the site, unminified." : "Built the assets and the site.");
