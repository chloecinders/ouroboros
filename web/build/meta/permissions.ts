import { execFile } from "node:child_process";
import path from "node:path";
import { promisify } from "node:util";

import { ROOT } from "./paths.ts";
import { decomment, item, literal, rust } from "./rust.ts";

const run = promisify(execFile);

const PERMISSIONS = path.join("src", "model", "permissions.rs");

interface Permission {
    upper: string;
    name: string;
    deprecated: boolean;
}

export async function permissions(): Promise<Permission[]> {
    const { stdout } = await run("cargo", ["metadata", "--format-version", "1"], {
        cwd: ROOT,
        maxBuffer: 128 * 1024 * 1024,
    });

    const metadata = JSON.parse(stdout) as { packages: { name: string; manifest_path: string }[] };
    const serenity = metadata.packages.find((entry) => entry.name === "serenity");

    if (!serenity) throw new Error("cargo metadata does not list serenity");

    const src = decomment(await rust(path.join(path.dirname(serenity.manifest_path), PERMISSIONS)));
    const table = item(src, /generate_permissions!\s*\{/);
    const found: Permission[] = [];

    for (const row of table.split(";")) {
        const match = /([A-Z][A-Z0-9_]*)\s*,\s*[a-z0-9_]+\s*,\s*"/.exec(row);

        if (!match) continue;

        found.push({
            upper: match[1]!,
            name: literal(row, row.indexOf('"', match.index + match[0].length - 1)).value,
            deprecated: /#\[\s*deprecated/.test(row),
        });
    }

    if (!found.length) throw new Error("serenity's permission table came back empty");

    return found;
}

export function permissionNames(table: Permission[], listed: string[], at: string): string[] {
    for (const one of listed)
        if (!table.some((permission) => permission.upper === one))
            throw new Error(`${at} names an unknown permission ${one}`);

    return table
        .filter((permission) => !permission.deprecated && listed.includes(permission.upper))
        .map((permission) => permission.name.toUpperCase().replaceAll(" ", "_"));
}
