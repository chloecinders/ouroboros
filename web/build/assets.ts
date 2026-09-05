import * as esbuild from "esbuild";
import { minify as minifyHtml } from "html-minifier-terser";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { jsx } from "./solid.ts";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const SOURCE = path.join(ROOT, "web", "apps");

const DEST = process.env.AEGIS_ASSET_OUT ?? path.join(ROOT, "target", "web-assets");

const STYLE = /<style>([\s\S]*?)<\/style>/;

const DEBUG = Boolean(process.env.AEGIS_DEBUG_BUILD);

const only = (built: esbuild.BuildResult<{ write: false }>) => built.outputFiles[0].text.trim();

async function sheet(entry: string): Promise<string> {
    return only(
        await esbuild.build({
            entryPoints: [entry],
            bundle: true,
            minify: !DEBUG,
            target: ["chrome100", "firefox100", "safari15", "edge100"],
            external: ["/fonts/*"],
            write: false,
        }),
    );
}

async function split(entry: string, out: string): Promise<string[]> {
    await fs.rm(out, { recursive: true, force: true });

    const built = await esbuild.build({
        entryPoints: [entry],
        outdir: out,
        bundle: true,
        splitting: true,
        format: "esm",
        entryNames: "[name]",
        chunkNames: "[name]-[hash]",
        minify: !DEBUG,
        platform: "browser",
        target: ["es2020"],
        legalComments: "none",
        external: ["/activity.js"],
        plugins: [jsx("dom")],
        metafile: true,
    });

    return Object.keys(built.metafile.outputs)
        .map((at) => path.basename(at))
        .sort();
}

async function fill(name: string, shell: string, painted: string): Promise<string> {
    if (!STYLE.test(shell)) throw new Error(`${name} has no <style> block for the build to fill`);

    const marked = shell.replace(STYLE, () => `<style>${""}__STYLE__</style>`);
    const built = DEBUG
        ? marked
        : await minifyHtml(marked, {
              collapseWhitespace: true,
              conservativeCollapse: true,
              removeComments: true,
              html5: true,
          });

    return built.replace("__STYLE__", () => painted);
}

async function app(name: string): Promise<{ html: string; chunks: string[] }> {
    const dir = path.join(SOURCE, name);
    const shell = await fs.readFile(path.join(dir, "page.html"), "utf8");

    const [painted, chunks] = await Promise.all([
        sheet(path.join(dir, "styles.css")),
        split(path.join(dir, "main.tsx"), path.join(DEST, name)),
    ]);

    return { html: await fill(name, shell, painted), chunks };
}

async function module(name: string): Promise<string> {
    return (
        await esbuild.build({
            entryPoints: [path.join(SOURCE, name)],
            bundle: true,
            minify: !DEBUG,
            format: "esm",
            platform: "browser",
            target: ["es2020"],
            legalComments: "none",
            write: false,
        })
    ).outputFiles[0].text;
}

const kb = (text: string | Uint8Array) => (Buffer.byteLength(text) / 1024).toFixed(1) + "kb";

async function main() {
    await fs.mkdir(DEST, { recursive: true });

    const entries = await fs.readdir(SOURCE, { withFileTypes: true });

    const apps: string[] = [];

    for (const entry of entries.filter((e) => e.isDirectory())) {
        try {
            await fs.access(path.join(SOURCE, entry.name, "page.html"));
            apps.push(entry.name);
        } catch {}
    }

    const modules = entries.filter((e) => e.isFile() && e.name.endsWith(".ts")).map((e) => e.name);

    if (apps.length === 0) throw new Error(`no app with a page.html in ${SOURCE}`);

    const wrote = async (out: string, name: string, after: string | Uint8Array) => {
        await fs.writeFile(path.join(DEST, out), after);
        console.log(`  ${name.padEnd(20)} -> ${out.padEnd(20)} ${kb(after).padStart(8)}`);
    };

    let split = 0;

    for (const name of apps) {
        const { html, chunks } = await app(name);

        await wrote(`${name}.html`, `${name}/`, html);

        for (const chunk of chunks) {
            const size = await fs.readFile(path.join(DEST, name, chunk));

            console.log(`  ${"".padEnd(20)}    ${`${name}/${chunk}`.padEnd(20)} ${kb(size).padStart(8)}`);
        }

        split += chunks.length;
    }

    for (const name of modules) await wrote(name.replace(/\.ts$/, ".js"), name, await module(name));

    console.log(`built ${apps.length} app(s) in ${split} chunk(s) and ${modules.length} module(s) into ${DEST}`);
}

await main();
