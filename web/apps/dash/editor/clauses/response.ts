import { spanOf } from "../duration.ts";
import { VERBS } from "../grammar.ts";
import type { Clause } from "../rule.ts";
import { bareChannel, idOf } from "../tokenise.ts";

export function readThreshold(clause: Clause): void {
    if (!clause.once("duplicate after clause")) return;

    const [times, joiner, ...written] = clause.rest;

    if (!times || !joiner) {
        const filled = times ? times.text + " in 10m" : "after 2 in 10m";

        clause.bad("incomplete after clause", "provide a valid threshold", filled);
        return;
    }

    if (joiner.text.toLowerCase() !== "in") {
        const filled = written.length ? "in" : "in " + joiner.text;

        clause.bad("expected in", "join the timeframe with in", filled);
        return;
    }

    if (!/^-?\d+$/.test(times.text)) {
        clause.bad("expected whole number", "provide a whole number", "5");
        return;
    }

    if (Number(times.text) < 2) {
        clause.bad("count below 2", "provide 2 or more", "2");
        return;
    }

    const secs = spanOf(written);

    if (secs === null) {
        clause.bad("not a duration", "provide a duration", "10m");
        return;
    }

    if (secs <= 0) {
        clause.bad("empty timeframe", "provide a timeframe", "10m");
        return;
    }

    clause.body.after = { count: Number(times.text), secs };
}

export function readAction(clause: Clause): void {
    if (!clause.once("duplicate then clause")) return;

    const [action, ...tail] = clause.rest;

    if (!action) {
        clause.bad("missing action", "provide an action", "then ban 7d");
        return;
    }

    const verb = action.text.toLowerCase();

    if (verb === "delete") {
        if (tail.length) {
            clause.bad("delete has no arguments");
            return;
        }

        clause.body.then = { verb, secs: null };

        return;
    }

    if (!VERBS.includes(verb)) {
        clause.bad("no action found", "provide a valid action", "ban");
        return;
    }

    let secs: number | null = null;

    if (tail.length) {
        if (!["ban", "mute"].includes(verb)) {
            clause.bad("only bans and mutes have durations");
            return;
        }

        secs = spanOf(tail);

        if (secs === null) {
            clause.bad("not a duration", "provide a duration", "7d");
            return;
        }
    }

    if (verb === "mute" && secs === null) clause.warn("mute with no duration uses the server default");

    clause.body.then = { verb, secs };
}

export function readDelete(clause: Clause): void {
    clause.body.delete = true;
}

export function readClear(clause: Clause): void {
    if (clause.seen.clear) clause.warn("line " + clause.seen.clear + " already clears");

    clause.seen.clear = clause.line;

    const amount = clause.rest[0];

    if (!amount) {
        clause.bad("missing days", "provide a number of days", "clear 1");
        return;
    }

    const bare = /^\d+$/.test(amount.text);
    const span = spanOf(clause.rest);
    const spanned = span !== null && (!bare || clause.rest.length > 1);

    if (!spanned && !/^-?\d+$/.test(amount.text)) {
        clause.bad("expected whole number", "provide a whole number", "5");
        return;
    }

    const days = spanned ? Math.trunc(span / 86400) : Number(amount.text);

    if (days < 0 || days > 7) {
        clause.bad("discord clears at most 7 days", "provide 0 to 7 days", days < 0 ? "0" : "7");
        return;
    }

    clause.body.clear = days;
}

export function readNotify(clause: Clause): void {
    if (clause.seen.notify) clause.warn("line " + clause.seen.notify + " already notifies");

    clause.seen.notify = clause.line;

    const token = clause.rest[0];

    if (!token) {
        clause.bad("missing channel", "provide a channel or none", "notify channel:<id>");
        return;
    }

    if (token.text.toLowerCase() === "none") {
        clause.body.notify = "none";
        return;
    }

    if (idOf(token, "role")) {
        clause.bad("found role, expected channel", "provide a channel", "channel:<id>");
        return;
    }

    const channel = idOf(token, "channel") || bareChannel(token);

    if (!channel) {
        clause.bad("expected channel:<id>", "provide a channel", "channel:<id>");
        return;
    }

    clause.body.notify = channel;
}

export function readReason(clause: Clause): void {
    if (!clause.once("duplicate reason clause")) return;

    const written = clause.text.slice(clause.head.text.length).trim();

    if (!written) {
        clause.bad("missing reason", "provide a reason", "reason scam bot");
        return;
    }

    clause.body.reason = written;
}
