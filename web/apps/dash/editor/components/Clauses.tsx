import { For, Show, createEffect, createMemo, onMount } from "solid-js";
import { createStore } from "solid-js/store";

import type { Entry } from "../../api.ts";
import { PERMISSIONS } from "../grammar.ts";
import { highlight } from "../highlight.ts";
import { useEditor } from "../state/editor.tsx";
import { Diags } from "./Diags.tsx";

interface Pop {
    open: boolean;
    kind: string | null;
    start: number;
    partial: string;
    items: Entry[];
    index: number;
    dismissed: boolean;
}

function sync(src: HTMLTextAreaElement, hl: HTMLElement, gutter: HTMLElement) {
    hl.scrollTop = src.scrollTop;
    hl.scrollLeft = src.scrollLeft;
    gutter.scrollTop = src.scrollTop;
}

export function Clauses() {
    const { session, editing, rules, managed } = useEditor();

    let src!: HTMLTextAreaElement;
    let hl!: HTMLPreElement;
    let gutter!: HTMLDivElement;
    let stack!: HTMLDivElement;
    let box!: HTMLDivElement;

    let charWidth = 8;

    const [pop, setPop] = createStore<Pop>({
        open: false,
        kind: null,
        start: 0,
        partial: "",
        items: [],
        index: 0,
        dismissed: false,
    });

    const open = () => Boolean(managed.open() || rules.open());
    const reading = () => editing.reading();
    const lines = createMemo(() => editing.draft().split("\n"));

    const marks = createMemo(() => {
        const byLine: Record<number, string> = {};

        for (const d of reading().diags) if (d.line) byLine[d.line] = d.level;

        return byLine;
    });

    const warns = () => reading().diags.length - reading().errors;

    const listing = (kind: string): Entry[] => {
        if (kind === "permission") return PERMISSIONS.map((permission) => ({ id: "", name: permission }));

        return kind === "role" ? (session.view()?.roles ?? []) : (session.view()?.channels ?? []);
    };

    function context() {
        if (!src || src.selectionStart !== src.selectionEnd) return null;

        const before = editing.draft().slice(0, editing.caret());
        const m = before.match(/(role|channel|permission):([\w-]*)$/i);

        if (!m) return null;

        return { kind: m[1].toLowerCase(), partial: m[2], start: editing.caret() - m[2].length };
    }

    function close() {
        setPop({ open: false, index: 0 });
    }

    function syncCompletion() {
        const ctx = context();

        if (!ctx) {
            close();
            setPop("dismissed", false);
            return;
        }

        if (pop.dismissed && pop.kind === ctx.kind && pop.start === ctx.start) return;

        const partial = ctx.partial.toLowerCase();
        const wanted = ctx.kind === "permission" ? partial.replace(/-/g, "_") : partial;
        const items = listing(ctx.kind).filter(
            (x) => !wanted || x.name.toLowerCase().includes(wanted) || x.id.startsWith(wanted),
        );

        setPop({
            open: true,
            dismissed: false,
            kind: ctx.kind,
            start: ctx.start,
            partial: ctx.partial,
            items,
            index: Math.min(pop.index, Math.max(items.length - 1, 0)),
        });
    }

    function place() {
        if (!pop.open || !box) return;

        const upto = editing.draft().slice(0, pop.start);
        const rows = upto.split("\n");
        const row = rows.length - 1;
        const col = rows[rows.length - 1].length;
        const pad = 10;

        let x = pad + col * charWidth - src.scrollLeft;
        let y = pad + (row + 1) * 24 - src.scrollTop;

        const frame = stack.getBoundingClientRect();
        const size = box.getBoundingClientRect();

        if (y + size.height > frame.height) y = Math.max(0, y - 24 - size.height);

        x = Math.max(0, Math.min(x, frame.width - size.width - 4));

        box.style.left = x + "px";
        box.style.top = y + "px";
    }

    function accept() {
        if (!pop.open || !pop.items.length) return;

        const pick = pop.items[pop.index];
        const value = pick.id || pick.name;
        const end = pop.start + pop.partial.length;
        const next = editing.draft().slice(0, pop.start) + value + editing.draft().slice(end);
        const at = pop.start + value.length;

        src.value = next;
        editing.setDraft(next);
        src.focus();
        src.setSelectionRange(at, at);
        editing.setCaret(at);

        close();
        setPop("dismissed", true);
        editing.check();
    }

    function move(step: number) {
        if (!pop.items.length) return;

        setPop("index", (pop.index + step + pop.items.length) % pop.items.length);

        const on = box && box.querySelector(".opt.on");

        if (on) on.scrollIntoView({ block: "nearest" });
    }

    function insertLine(text: string) {
        const at = src.selectionStart;
        const before = editing.draft().slice(0, at);
        const after = editing.draft().slice(at);
        const atLineStart = before === "" || before.endsWith("\n");
        const insert = (atLineStart ? "" : "\n") + text + (after.startsWith("\n") || after === "" ? "" : "\n");
        const next = before + insert + after;

        src.value = next;
        editing.setDraft(next);
        src.focus();
        src.setSelectionRange(at + insert.length, at + insert.length);
        editing.setCaret(at + insert.length);
        editing.check();
    }

    onMount(() => {
        session.surface.source = src;

        const probe = document.createElement("span");
        const style = getComputedStyle(src);

        probe.textContent = "0".repeat(40);
        probe.style.cssText =
            "position:absolute;visibility:hidden;white-space:pre;font-family:" +
            style.fontFamily +
            ";font-size:" +
            style.fontSize +
            ";font-variant-ligatures:none";

        document.body.appendChild(probe);
        charWidth = probe.getBoundingClientRect().width / 40;
        probe.remove();
    });

    createEffect(() => {
        const text = editing.draft();

        if (src.value === text) return;

        src.value = text;
        src.setSelectionRange(0, 0);
    });

    createEffect(() => {
        editing.draft();
        editing.caret();
        syncCompletion();
    });

    createEffect(() => {
        editing.draft();
        sync(src, hl, gutter);
    });

    createEffect(() => {
        pop.open;
        pop.items;
        pop.index;
        place();
    });

    function keydown(e: KeyboardEvent) {
        if (pop.open) {
            if (e.key === "ArrowDown") {
                e.preventDefault();
                move(1);
                return;
            }

            if (e.key === "ArrowUp") {
                e.preventDefault();
                move(-1);
                return;
            }

            if (e.key === "Enter" || e.key === "Tab") {
                e.preventDefault();
                accept();
                return;
            }

            if (e.key === "Escape") {
                e.preventDefault();
                setPop("dismissed", true);
                close();
                return;
            }
        }

        if (e.key === "Tab") {
            e.preventDefault();
            insertLine("");
        }

        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            e.preventDefault();

            if (managed.open()) managed.keep();
            else rules.commit();
        }
    }

    return (
        <>
            <div class="editbar">
                <span class="editbar__label">clauses</span>
                <span class="editbar__counts">
                    <Show when={reading().errors}>
                        <span class="editbar__errors">
                            {reading().errors} error{reading().errors > 1 ? "s" : ""}
                        </span>
                    </Show>
                    <Show when={warns() > 0}>
                        <span class="editbar__warnings">
                            {warns()} warning{warns() > 1 ? "s" : ""}
                        </span>
                    </Show>
                </span>
            </div>

            <div>
                <div class="code">
                    <div class="code__gutter" ref={gutter}>
                        <Show when={open()}>
                            <For each={lines()}>
                                {(text, index) => {
                                    const n = () => index() + 1;
                                    const level = () => marks()[n()];
                                    const mark = () =>
                                        level() === "error"
                                            ? "×"
                                            : level() === "warn"
                                              ? "!"
                                              : text.trim() === ""
                                                ? ""
                                                : "•";

                                    return (
                                        <div
                                            class={[
                                                "code__line",
                                                level() === "error"
                                                    ? "code__line--err"
                                                    : level() === "warn"
                                                      ? "code__line--warn"
                                                      : "",
                                                n() === editing.line() ? "code__line--cur" : "",
                                            ]
                                                .filter(Boolean)
                                                .join(" ")}
                                        >
                                            <span>{n()}</span>
                                            <span class="code__mark">{mark()}</span>
                                        </div>
                                    );
                                }}
                            </For>
                        </Show>
                    </div>

                    <div class="code__stack" ref={stack}>
                        <pre
                            class="code__paint"
                            ref={hl}
                            innerHTML={open() ? highlight(editing.draft(), reading().diags) + "\n" : ""}
                        />

                        <textarea
                            class="code__input"
                            ref={src}
                            spellcheck={false}
                            autocomplete="off"
                            wrap="off"
                            disabled={!open()}
                            onInput={(e) => {
                                editing.setDraft(e.currentTarget.value);
                                editing.setCaret(e.currentTarget.selectionStart);
                                session.say(null);
                                editing.check();
                            }}
                            onScroll={() => sync(src, hl, gutter)}
                            onKeyUp={(e) => editing.setCaret(e.currentTarget.selectionStart)}
                            onClick={(e) => editing.setCaret(e.currentTarget.selectionStart)}
                            onKeyDown={keydown}
                            onBlur={close}
                        />

                        <div class="complete" ref={box} hidden={!pop.open}>
                            <Show
                                when={pop.items.length}
                                fallback={<div class="complete__none">no {pop.kind} found</div>}
                            >
                                <For each={pop.items}>
                                    {(x, i) => (
                                        <div
                                            class={
                                                i() === pop.index
                                                    ? "complete__opt complete__opt--on"
                                                    : "complete__opt"
                                            }
                                            onMouseDown={(e) => {
                                                e.preventDefault();
                                                setPop("index", i());
                                                accept();
                                            }}
                                        >
                                            <span class="complete__name">
                                                {pop.kind === "channel" ? "#" + x.name : x.name}
                                            </span>
                                            <span class="complete__id">{x.id || pop.kind}</span>
                                        </div>
                                    )}
                                </For>
                            </Show>
                        </div>
                    </div>
                </div>

                <Diags />
            </div>
        </>
    );
}
