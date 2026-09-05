import type { Accessor } from "solid-js";
import { batch, createMemo, createSignal, onCleanup } from "solid-js";

import type { Error, Reading } from "../../api.ts";
import { API, call } from "../../api.ts";
import type { Count } from "../evaluate.ts";
import { evaluate } from "../evaluate.ts";
import { KEYWORDS } from "../grammar.ts";
import { parse } from "../parse.ts";
import type { Diag, Parsed, Part } from "../rule.ts";
import { tokenise } from "../tokenise.ts";
import type { Preview } from "./preview.ts";
import type { Session } from "./session.ts";

export interface Read {
    parsed: Parsed;
    diags: Diag[];
    errors: number;
    checked: boolean;
}

interface Picked {
    keyword: string;
    over: string | null;
}

export interface Editing {
    draft: Accessor<string>;
    setDraft: (text: string) => void;
    load: (text: string, block: Part) => void;
    caret: Accessor<number>;
    setCaret: (at: number) => void;
    line: Accessor<number>;
    vocab: Accessor<string | null>;
    pick: (keyword: string) => void;
    unpick: () => void;
    reading: Accessor<Read>;
    result: Accessor<Count>;
    blame: (detail: Error | undefined) => void;
    check: () => void;
}

export function createEditing(session: Session, preview: Preview): Editing {
    const [draft, setDraft] = createSignal("");
    const [part, setPart] = createSignal<Part>(session.part);
    const [caret, setCaret] = createSignal(0);
    const [verdict, setVerdict] = createSignal<{ value: Reading; of: string } | null>(null);
    const [picked, setPicked] = createSignal<Picked | null>(null);
    const [dismissed, setDismissed] = createSignal<string | null>(null);

    const line = createMemo(() => draft().slice(0, caret()).split("\n").length);

    const word = createMemo(() => {
        const at = draft().split("\n")[line() - 1] || "";
        const first = tokenise(at)[0]?.text;

        if (!first) return null;

        const lowered = first.toLowerCase();

        if (KEYWORDS.includes(lowered)) return lowered;

        if (lowered.length >= 2) {
            const near = KEYWORDS.filter((keyword) => keyword.startsWith(lowered));

            if (near.length === 1) return near[0];
        }

        return null;
    });

    const vocab = createMemo(() => {
        const chosen = picked();

        if (chosen && chosen.over === word()) return chosen.keyword;

        const near = word();

        return near && near !== dismissed() ? near : null;
    });

    const reading = createMemo<Read>(() => {
        const source = draft();
        const parsed = parse(source, part());
        const checked = verdict();
        const from = checked && checked.of === source ? checked.value : null;

        if (!from || from.ok || !from.error)
            return { parsed, diags: parsed.diags, errors: parsed.errors, checked: !!from };

        const failed = from.error;
        const at = typeof failed.start === "number" ? source.slice(0, failed.start).split("\n").length : 0;
        const diags = parsed.diags.slice();

        if (!diags.some((one) => one.line === at && one.level === "error"))
            diags.push({ line: at, level: "error", msg: failed.problem });

        return { parsed, diags, errors: parsed.errors + 1, checked: true };
    });

    const result = createMemo(() => evaluate(reading().parsed.body, preview.observed()));

    let checking: ReturnType<typeof setTimeout> | undefined;

    async function checked() {
        const asked = draft();
        const answer = await call<Reading>(API.check, "POST", { source: asked, part: part() });

        if (asked !== draft()) return;

        if (answer.error === "anonymous") return session.setRefused("anonymous");

        if (!answer.error) setVerdict({ value: answer.value, of: asked });
    }

    function check() {
        clearTimeout(checking);

        checking = setTimeout(checked, 450);
    }

    onCleanup(() => clearTimeout(checking));

    return {
        draft,
        setDraft,
        load: (text, block) =>
            batch(() => {
                setPart(block);
                setDraft(text);
                setCaret(0);
                setVerdict(null);
            }),
        caret,
        setCaret,
        line,
        vocab,
        pick: (keyword) => setPicked({ keyword, over: word() }),
        unpick: () =>
            batch(() => {
                setPicked(null);
                setDismissed(word());
            }),
        reading,
        result,
        blame: (detail) => setVerdict({ value: { ok: false, error: detail }, of: draft() }),
        check,
    };
}
