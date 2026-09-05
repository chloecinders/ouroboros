import { spanOf } from "../duration.ts";
import { CMPS, MEASURES, RECORD, RETIRED, SOURCES } from "../grammar.ts";
import type { Clause } from "../rule.ts";
import { readMatcher } from "../tokenise.ts";

export function readSources(clause: Clause): void {
    if (clause.rest.length === 0) {
        clause.bad("no source found", "provide a valid source", "on content");
        return;
    }

    for (const token of clause.rest) {
        const source = token.text.toLowerCase();

        if (!SOURCES.includes(source)) {
            clause.bad("no source found", "provide a valid source", "content");
            continue;
        }

        if (!clause.body.sources.includes(source)) clause.body.sources.push(source);
    }
}

export function readPattern(clause: Clause): void {
    const written = clause.text.slice(clause.head.text.length).trim();

    if (!written) {
        const filled = clause.word === "never" ? 'never "nitro giveaway rules"' : 'match "free nitro"';

        clause.bad("missing pattern", "provide a pattern", filled);
        return;
    }

    const read = readMatcher(written);

    if (read.error !== undefined) {
        clause.bad(read.error);
        return;
    }

    (clause.word === "match" ? clause.body.match : clause.body.never).push(read);
}

export function readWhen(clause: Clause): void {
    const [subject, middle, bound, ...tail] = clause.rest;

    if (!subject || !middle || !bound) {
        const filled = subject ? subject.text + " > 5" : "when mentions > 5";

        clause.bad("incomplete when clause", "provide a valid when clause", filled);
        return;
    }

    const measure = RETIRED[subject.text.toLowerCase()] || subject.text.toLowerCase();

    if (measure === "account") {
        const side = middle.text.toLowerCase();

        if (side !== "younger" && side !== "older") {
            const filled = bound.text.toLowerCase() === "than" ? "younger" : "younger than";

            clause.bad("expected younger or older", "provide younger or older", filled);
            return;
        }

        if (bound.text.toLowerCase() !== "than") {
            const filled = tail.length ? "than" : "than " + bound.text;

            clause.bad("expected than", "join the age with than", filled);
            return;
        }

        const secs = spanOf(tail);

        if (secs === null) {
            clause.bad("not a duration", "provide a duration", "7d");
            return;
        }

        clause.body.when.push({
            measure: "account",
            dir: side as "younger" | "older",
            secs,
            line: clause.line,
            text: clause.text,
        });

        return;
    }

    const onRecord = RECORD.includes(measure);

    if (!onRecord && !MEASURES.includes(measure)) {
        clause.bad("no measure found", "provide a valid measure", "mentions");
        return;
    }

    if (!CMPS.includes(middle.text)) {
        const help = "compare with an operator";

        clause.bad("expected >, <, >= or <=", help, onRecord ? ">=" : ">");
        return;
    }

    if (!/^-?\d+$/.test(bound.text)) {
        clause.bad("expected whole number", "provide a whole number", "5");
        return;
    }

    const val = Number(bound.text);

    if (onRecord && val < 0) {
        clause.bad("negative count", "provide 0 or more", bound.text.replace(/^-+/, ""));
        return;
    }

    let within: number | null = null;

    if (onRecord && tail.length) {
        const [joiner, ...written] = tail;

        if (joiner.text.toLowerCase() !== "in") {
            const filled = written.length ? "in" : "in " + joiner.text;

            clause.bad("expected in", "join the window with in", filled);
            return;
        }

        within = spanOf(written);

        if (within === null) {
            clause.bad("not a duration", "provide a duration", "30d");
            return;
        }

        if (within <= 0) {
            clause.bad("empty window", "provide a window", "30d");
            return;
        }
    }

    if (onRecord && within === null) clause.warn(measure + " with no window counts their whole log");

    clause.body.when.push({ measure, cmp: middle.text, val, within, line: clause.line, text: clause.text });
}
