import * as esbuild from "esbuild";
import { marked } from "marked";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { jsx } from "./solid.ts";
import { sheet } from "./meta.ts";
import { titleCase } from "../site/src/wiki/commands.ts";
import type { Prosed } from "../site/src/render.tsx";
import { GENERAL } from "../site/src/wiki/general.ts";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const SITE = path.join(ROOT, "web", "site");
const SOURCE = path.join(SITE, "src");
const STAGE = path.join(ROOT, "target", "site-stage");
const PROSE = path.join(SITE, "wiki", "commands");

const BUILT = path.join(ROOT, "target", "site-build", "render.mjs");

const exists = async (at: string) => {
    try {
        await fs.access(at);
        return true;
    } catch {
        return false;
    }
};

async function renderer(): Promise<typeof import("../site/src/render.tsx")> {
    await esbuild.build({
        entryPoints: [path.join(SOURCE, "render.tsx")],
        outfile: BUILT,
        bundle: true,
        format: "esm",
        platform: "node",
        target: ["node20"],
        plugins: [jsx("ssr")],
    });

    return import(pathToFileURL(BUILT).href);
}

async function prose(): Promise<(name: string) => string> {
    const written = new Map<string, string>();

    if (!(await exists(PROSE))) return () => "";

    for (const name of await fs.readdir(PROSE)) {
        if (!name.endsWith(".html")) continue;

        written.set(path.basename(name, ".html"), (await fs.readFile(path.join(PROSE, name), "utf8")).trim());
    }

    return (name: string) => written.get(name) || "";
}

async function general(updated: string): Promise<Prosed[]> {
    const written: Prosed[] = [];

    for (const name of GENERAL) {
        const at = path.join(SITE, "wiki", "general", `${name}.md`);

        if (!(await exists(at))) throw new Error(`${at} is missing, but the wiki lists "${name}" as a page.`);

        const text = (await fs.readFile(at, "utf8")).replaceAll("{{updated}}", updated);

        written.push({ name, title: titleCase(name), html: (await marked.parse(text)).trim() });
    }

    return written;
}

async function main() {
    const updated = new Date().toLocaleDateString("en-US", { year: "numeric", month: "long", day: "numeric" });
    const [{ pages }, sheeted, look, prosed] = await Promise.all([renderer(), sheet(), prose(), general(updated)]);
    const written = pages({ sheet: sheeted, prose: look, general: prosed, updated });
    await fs.rm(STAGE, { recursive: true, force: true });

    for (const page of written) {
        const at = path.join(STAGE, page.file);

        await fs.mkdir(path.dirname(at), { recursive: true });
        await fs.writeFile(at, page.html);
    }

    await esbuild.build({
        entryPoints: [path.join(SITE, "styles.css")],
        outfile: path.join(STAGE, "styles.css"),
        bundle: true,
        target: ["chrome100", "firefox100", "safari15", "edge100"],
        external: ["/fonts/*"],
    });

    await fs.copyFile(path.join(SITE, "favicon.webp"), path.join(STAGE, "favicon.webp"));

    if (await exists(path.join(SITE, "images")))
        await fs.cp(path.join(SITE, "images"), path.join(STAGE, "images"), { recursive: true });

    console.log(`Rendered ${written.length} page(s), and staged the stylesheet, the favicon and images.`);
}

await main();
