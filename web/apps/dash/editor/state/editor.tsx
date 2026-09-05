import { useBeforeLeave, useSearchParams } from "@solidjs/router";
import type { JSX } from "solid-js";
import { createContext, createEffect, onCleanup, onMount, useContext } from "solid-js";

import { ACTIVITY } from "../../api.ts";
import { discard } from "../ask.ts";
import type { Editing } from "./editing.ts";
import { createEditing } from "./editing.ts";
import type { Focus } from "./focus.ts";
import { createFocus } from "./focus.ts";
import { load } from "./load.ts";
import type { Managed } from "./managed.ts";
import { createManaged } from "./managed.ts";
import type { Preview } from "./preview.ts";
import { createPreview } from "./preview.ts";
import type { Rules } from "./rules.ts";
import { createRules } from "./rules.ts";
import type { Mode, Session } from "./session.ts";
import { createSession } from "./session.ts";

export interface Editor {
    session: Session;
    focus: Focus;
    preview: Preview;
    editing: Editing;
    rules: Rules;
    managed: Managed;
}

export interface Want {
    rule?: string;
    managed?: string;
}

export function createEditor(mode: Mode, guild: string): Editor {
    const session = createSession(mode, guild);
    const focus = createFocus();
    const preview = createPreview(session);
    const editing = createEditing(session, preview);
    const rules = createRules(session, focus, editing);
    const managed = createManaged(session, focus, editing, rules);

    return { session, focus, preview, editing, rules, managed };
}

const Context = createContext<Editor>();

export function useEditor(): Editor {
    const editor = useContext(Context);

    if (!editor) throw new Error("the editor is only usable under an EditorProvider");

    return editor;
}

const first = (value: string | string[] | undefined) => (Array.isArray(value) ? value[0] : value);

function asked(): Want {
    const [search] = useSearchParams();

    return { rule: first(search.rule), managed: first(search.managed) };
}

function holdTitle() {
    const title = document.title;

    onCleanup(() => {
        document.title = title;
    });
}

function guardExit(it: Editor) {
    const leaving = (e: BeforeUnloadEvent) => {
        if (!it.rules.dirty()) return;

        e.preventDefault();
        e.returnValue = "";
    };

    addEventListener("beforeunload", leaving);
    onCleanup(() => removeEventListener("beforeunload", leaving));

    useBeforeLeave((e) => {
        if (!it.rules.dirty()) return;

        const open = it.rules.open();

        if (!open) return;

        e.preventDefault();

        discard(open.name).then((yes) => {
            if (yes) e.retry(true);
        });
    });
}

function trackUrl(it: Editor) {
    if (ACTIVITY) return;

    createEffect(() => {
        if (!it.session.ready()) return;

        const managed = it.focus.managed();
        const rule = it.focus.rule();
        const query = managed
            ? "?managed=" + encodeURIComponent(managed)
            : rule
              ? "?rule=" + encodeURIComponent(rule)
              : "";

        history.replaceState(null, "", location.pathname + query);
    });
}

export function EditorProvider(props: { mode: Mode; guild: string; children: JSX.Element }) {
    const editor = createEditor(props.mode, props.guild);
    const want = asked();

    holdTitle();
    guardExit(editor);
    trackUrl(editor);

    onMount(() => load(editor, want));

    return <Context.Provider value={editor}>{props.children}</Context.Provider>;
}
