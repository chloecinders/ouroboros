import type { Accessor, Setter } from "solid-js";
import { createSignal } from "solid-js";

import type { Answer, Refusal, View } from "../../api.ts";
import { API, wording } from "../../api.ts";
import { CATCHES, TEMPLATE } from "../grammar.ts";
import type { Part } from "../rule.ts";

export type Mode = "guild" | "authored";

export interface Said {
    text: string;
    kind: string;
}

export interface Surface {
    name?: HTMLHeadingElement;
    source?: HTMLTextAreaElement;
}

interface Where {
    view: string;
    all: string;
    one: (rule: string) => string;
    offers: string;
    offer: (rule: string) => string;
}

export interface Session {
    mode: Mode;
    authored: boolean;
    guild: string;
    where: Where;
    part: Part;
    seed: string;
    view: Accessor<View | null>;
    setView: Setter<View | null>;
    ready: Accessor<boolean>;
    setReady: Setter<boolean>;
    refused: Accessor<Refusal | null>;
    setRefused: Setter<Refusal | null>;
    busy: Accessor<boolean>;
    during: <T>(run: () => Promise<T>) => Promise<T>;
    said: Accessor<Said | null>;
    say: (text: string | null, kind?: string) => void;
    surface: Surface;
}

export const trouble = <T,>(answer: Answer<T>): string =>
    answer.detail ? answer.detail.problem : wording(answer.error);

export function createSession(mode: Mode, guild: string): Session {
    const authored = mode === "authored";

    const [view, setView] = createSignal<View | null>(null);
    const [ready, setReady] = createSignal(false);
    const [refused, setRefused] = createSignal<Refusal | null>(null);
    const [busy, setBusy] = createSignal(false);
    const [said, setSaid] = createSignal<Said | null>(null);

    async function during<T>(run: () => Promise<T>): Promise<T> {
        setBusy(true);

        try {
            return await run();
        } finally {
            setBusy(false);
        }
    }

    return {
        mode,
        authored,
        guild,
        where: {
            view: API.guild(guild),
            all: authored ? API.authoring : API.rules(guild),
            one: (rule) => (authored ? API.authored(rule) : API.rule(guild, rule)),
            offers: API.managed_rules(guild),
            offer: (rule) => API.managed(guild, rule),
        },
        part: authored ? "detection" : "whole",
        seed: authored ? CATCHES : TEMPLATE,
        view,
        setView,
        ready,
        setReady,
        refused,
        setRefused,
        busy,
        during,
        said,
        say: (text, kind) => setSaid(text ? { text, kind: kind || "" } : null),
        surface: {},
    };
}
