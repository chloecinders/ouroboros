import * as esbuild from "esbuild";
import { minify as minifyHtml } from "html-minifier-terser";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const STAGE = path.join(ROOT, "target", "site-stage");
const DIST = path.join(ROOT, "web", "site", "dist");

const DEBUG = Boolean(process.env.AEGIS_DEBUG_BUILD);

async function* walk(dir: string): AsyncGenerator<string> {
    for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
        const at = path.join(dir, entry.name);

        if (entry.isDirectory()) yield* walk(at);
        else yield at;
    }
}

async function emit(at: string, wanted: Buffer): Promise<boolean> {
    try {
        if ((await fs.readFile(at)).equals(wanted)) return false;
    } catch { }

    await fs.mkdir(path.dirname(at), { recursive: true });
    await fs.writeFile(at, wanted);

    return true;
}

async function prune(keeping: Set<string>): Promise<number> {
    let removed = 0;

    try {
        await fs.access(DIST);
    } catch {
        return removed;
    }

    for await (const file of walk(DIST)) {
        if (keeping.has(path.relative(DIST, file))) continue;

        await fs.rm(file);

        removed += 1;
    }

    return removed;
}

async function main() {
    try {
        await fs.access(STAGE);
    } catch {
        throw new Error(`${STAGE} does not exist. Build using pnpm run build:site`);
    }

    const staged = new Set<string>();

    let before = 0;
    let after = 0;
    let touched = 0;
    let written = 0;

    for await (const file of walk(STAGE)) {
        const kind = path.extname(file);
        const relative = path.relative(STAGE, file);
        const source = await fs.readFile(file);

        staged.add(relative);

        let body = source;

        if (!DEBUG && (kind === ".html" || kind === ".css")) {
            const text = source.toString("utf8");

            body = Buffer.from(
                kind === ".css"
                    ? (await esbuild.transform(text, { loader: "css", minify: true })).code
                    : await minifyHtml(text, {
                        collapseWhitespace: true,
                        conservativeCollapse: true,
                        removeComments: true,
                        html5: true,
                        minifyCSS: true,
                        minifyJS: true,
                    }),
            );

            before += source.byteLength;
            after += body.byteLength;
            touched += 1;
        }

        if (await emit(path.join(DIST, relative), body)) written += 1;
    }

    const removed = await prune(staged);
    const kb = (bytes: number) => (bytes / 1024).toFixed(1) + "kb";

    console.log(DEBUG ? `Copied ${staged.size} file(s) unminified.` : `Minified ${touched} file(s), ${kb(before)} -> ${kb(after)}.`);
    console.log(`${written} changed file(s) written into web/site/dist, ${removed} stale one(s) removed.`);
}

await main();
