import fs from "node:fs/promises";
import path from "node:path";

const IDENT = /[A-Za-z0-9_]/;

export async function rust(at: string): Promise<string> {
    return (await fs.readFile(at, "utf8")).replaceAll("\r\n", "\n");
}

export async function sources(from: string): Promise<string[]> {
    const found: string[] = [];

    for (const entry of await fs.readdir(from, { withFileTypes: true })) {
        const at = path.join(from, entry.name);

        if (entry.isDirectory()) found.push(...(await sources(at)));
        else if (entry.name.endsWith(".rs")) found.push(at);
    }

    return found;
}

export function literal(src: string, from: number): { value: string; end: number } {
    let at = from + 1;
    let value = "";

    while (at < src.length && src[at] !== '"') {
        if (src[at] !== "\\") {
            value += src[at];
            at += 1;

            continue;
        }

        const escape = src[at + 1];

        at += 2;

        if (escape === "n") value += "\n";
        else if (escape === "t") value += "\t";
        else if (escape === "r") value += "\r";
        else if (escape === "0") value += "\0";
        else if (escape === '"' || escape === "'" || escape === "\\") value += escape;
        else if (escape === "\n") while (at < src.length && " \t\n".includes(src[at]!)) at += 1;
        else if (escape === "x") {
            value += String.fromCharCode(parseInt(src.slice(at, at + 2), 16));
            at += 2;
        } else if (escape === "u") {
            const close = src.indexOf("}", at);

            value += String.fromCodePoint(parseInt(src.slice(at + 1, close), 16));
            at = close + 1;
        } else throw new Error(`unknown escape \\${escape} in a Rust string literal`);
    }

    return { value, end: at + 1 };
}

function raw(src: string, from: number): number {
    let at = from + 1;
    let hashes = 0;

    while (src[at] === "#") {
        hashes += 1;
        at += 1;
    }

    if (src[at] !== '"') return -1;

    const close = '"' + "#".repeat(hashes);
    const end = src.indexOf(close, at + 1);

    if (end < 0) throw new Error("unterminated raw string");

    return end + close.length;
}

function character(src: string, from: number): number {
    if (src[from + 1] === "\\") return src[from + 3] === "'" ? from + 4 : -1;

    return src[from + 2] === "'" ? from + 3 : -1;
}

export function decomment(src: string): string {
    let out = "";
    let at = 0;

    while (at < src.length) {
        const here = src[at]!;

        if (here === '"') {
            const { end } = literal(src, at);

            out += src.slice(at, end);
            at = end;

            continue;
        }

        if (here === "r" && !IDENT.test(src[at - 1] ?? "")) {
            const end = raw(src, at);

            if (end > 0) {
                out += src.slice(at, end);
                at = end;

                continue;
            }
        }

        if (here === "'") {
            const end = character(src, at);

            if (end > 0) {
                out += src.slice(at, end);
                at = end;

                continue;
            }
        }

        if (here === "/" && src[at + 1] === "/") {
            while (at < src.length && src[at] !== "\n") {
                out += " ";
                at += 1;
            }

            continue;
        }

        if (here === "/" && src[at + 1] === "*") {
            let depth = 0;

            while (at < src.length) {
                if (src[at] === "/" && src[at + 1] === "*") {
                    depth += 1;
                    out += "  ";
                    at += 2;

                    continue;
                }

                if (src[at] === "*" && src[at + 1] === "/") {
                    depth -= 1;
                    out += "  ";
                    at += 2;

                    if (!depth) break;

                    continue;
                }

                out += src[at] === "\n" ? "\n" : " ";
                at += 1;
            }

            continue;
        }

        out += here;
        at += 1;
    }

    return out;
}

export function block(src: string, from: number): number {
    const open = src[from]!;
    const close = ({ "{": "}", "[": "]", "(": ")", "<": ">" } as Record<string, string>)[open];

    if (!close) throw new Error(`${open} does not open a block`);

    let depth = 0;
    let at = from;

    while (at < src.length) {
        const here = src[at]!;

        if (here === '"') {
            at = literal(src, at).end;

            continue;
        }

        if (here === "r" && !IDENT.test(src[at - 1] ?? "")) {
            const end = raw(src, at);

            if (end > 0) {
                at = end;

                continue;
            }
        }

        if (here === "'") {
            const end = character(src, at);

            if (end > 0) {
                at = end;

                continue;
            }
        }

        if (here === open) depth += 1;
        else if (here === close) {
            depth -= 1;

            if (!depth) return at + 1;
        }

        at += 1;
    }

    throw new Error(`unterminated ${open}`);
}

export function inner(src: string, from: number): string {
    return src.slice(from + 1, block(src, from) - 1);
}

export function pieces(text: string, sep: string): string[] {
    const found: string[] = [];

    let depth = 0;
    let start = 0;
    let at = 0;

    while (at < text.length) {
        const here = text[at]!;

        if (here === '"') {
            at = literal(text, at).end;

            continue;
        }

        if (here === "'") {
            const end = character(text, at);

            if (end > 0) {
                at = end;

                continue;
            }
        }

        if ("{[(<".includes(here)) depth += 1;
        else if ("}])>".includes(here)) depth -= 1;
        else if (here === sep && !depth) {
            found.push(text.slice(start, at));
            start = at + 1;
        }

        at += 1;
    }

    found.push(text.slice(start));

    return found.map((part) => part.trim()).filter((part) => part.length > 0);
}

export function arms(body: string, holder: string): Map<string, string> {
    const found = new Map<string, string>();
    const arm = new RegExp(`((?:${holder}::[A-Za-z0-9_]+\\s*\\|\\s*)*${holder}::[A-Za-z0-9_]+)\\s*=>\\s*"`, "g");

    for (const match of body.matchAll(arm)) {
        const { value } = literal(body, match.index + match[0].length - 1);

        for (const variant of match[1]!.split("|")) found.set(variant.trim().slice(holder.length + 2), value);
    }

    return found;
}

export function item(src: string, head: RegExp, opener = "{"): string {
    const match = head.exec(src);

    if (!match) throw new Error(`could not find ${head.source} in the Rust sources`);

    return inner(src, src.indexOf(opener, match.index + match[0].length - 1));
}
