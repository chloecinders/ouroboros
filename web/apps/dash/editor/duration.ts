import type { Token } from "./tokenise.ts";

const UNITS: Record<string, number> = { s: 1, m: 60, h: 3600, d: 86400, w: 604800, M: 2592000, y: 31536000 };

export const DUR = /^\d+[smhdwMy]$/;

export function parseDuration(raw: string): number | null {
    if (/^\d+$/.test(raw)) return Number(raw);

    const unit = raw.slice(-1);
    const amount = raw.slice(0, -1);

    if (!(unit in UNITS) || !/^\d+$/.test(amount)) return null;

    return Number(amount) * UNITS[unit];
}

export function spanOf(tokens: Token[]): number | null {
    if (tokens.length === 1) return parseDuration(tokens[0].text);
    if (tokens.length !== 2) return null;

    const amount = tokens[0].text;
    const singular = String(tokens[1].text || "")
        .toLowerCase()
        .replace(/s$/, "");
    const initial: string | undefined = {
        second: "s",
        minute: "m",
        hour: "h",
        day: "d",
        week: "w",
        month: "M",
        year: "y",
    }[singular];

    if (!initial || !/^\d+$/.test(amount)) return null;

    return Number(amount) * UNITS[initial];
}

export function showDuration(secs: number): string {
    const units: [string, number][] = [
        ["w", 604800],
        ["d", 86400],
        ["h", 3600],
        ["m", 60],
        ["s", 1],
    ];

    let left = secs;
    let out = "";

    for (const [label, size] of units) {
        if (left >= size) {
            out += Math.floor(left / size) + label;
            left %= size;
        }
    }

    return out || "0s";
}
