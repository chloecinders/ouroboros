import { For, Show, createEffect, onMount } from "solid-js";

import { useEditor } from "../state/editor.tsx";
import type { Taken } from "../state/managed.ts";

function Name() {
    const { session, rules, managed } = useEditor();

    let box!: HTMLHeadingElement;

    onMount(() => {
        session.surface.name = box;
    });

    createEffect(() => {
        const open = managed.open() || rules.open();
        const wanted = open ? open.name : "no rules";

        if (box.textContent !== wanted && document.activeElement !== box) box.textContent = wanted;
    });

    return (
        <h1
            class="rulehead__title"
            ref={box}
            contenteditable={managed.open() ? "false" : "true"}
            spellcheck={false}
            onInput={(e) => {
                if (!rules.open()) return;

                rules.rename(e.currentTarget.textContent?.trim() || "untitled");
                session.say(null);
            }}
            onKeyDown={(e) => {
                if (e.key === "Enter") {
                    e.preventDefault();
                    e.currentTarget.blur();
                }
            }}
        >
            loading
        </h1>
    );
}

function Ladder() {
    const { session, editing, rules, managed } = useEditor();

    const at = () => (managed.open() ? managed.open()?.mode : rules.open()?.mode);
    const off = (mode: string) => {
        const open = managed.open();

        if (open) return !open.mode;

        return !rules.open() || (mode !== "disabled" && Boolean(editing.reading().errors));
    };

    function pick(mode: string) {
        const open = managed.open();

        if (open) {
            if (!open.mode) return;

            managed.enable(mode);
            session.say(null);

            return;
        }

        if (!rules.open()) return;

        if (mode !== "disabled" && editing.reading().errors) {
            session.say("rule has errors", "bad");
            return;
        }

        rules.enable(mode);
        session.say(null);
    }

    return (
        <div class="ladder">
            <For each={["disabled", "active"]}>
                {(mode) => (
                    <button
                        data-mode={mode}
                        class={[
                            "ladder__mode",
                            at() === mode ? "ladder__mode--on" : "",
                            off(mode) ? "ladder__mode--blocked" : "",
                        ]
                            .filter(Boolean)
                            .join(" ")}
                        onClick={() => pick(mode)}
                    >
                        {mode}
                    </button>
                )}
            </For>
        </div>
    );
}

function Status() {
    const { session, rules, managed } = useEditor();

    const pending = () => (managed.open() ? managed.unsaved() : rules.dirty());
    const kind = () => (session.said() ? session.said()?.kind : pending() ? "dirty" : "");
    const text = () => (session.said() ? session.said()?.text : pending() ? "unsaved changes" : "");

    return <span class={kind() ? "rulehead__status rulehead__status--" + kind() : "rulehead__status"}>{text()}</span>;
}

function Own() {
    const { session, rules } = useEditor();

    return (
        <div class="rulehead__acts">
            <button class="editor__button" disabled={!rules.dirty() || session.busy()} onClick={rules.commit}>
                save
            </button>
            <button
                class="editor__button editor__button--line"
                disabled={!rules.dirty() || session.busy()}
                onClick={rules.revert}
            >
                revert
            </button>
            <button
                class="editor__button editor__button--danger"
                disabled={!rules.open() || session.busy()}
                onClick={rules.remove}
            >
                delete
            </button>
        </div>
    );
}

function Shared(props: { managed: Taken }) {
    const { session, managed } = useEditor();

    return (
        <div class="rulehead__acts">
            <Show
                when={props.managed.mode}
                fallback={
                    <button
                        class="editor__button"
                        disabled={session.busy() || props.managed.offered === "disabled"}
                        onClick={managed.subscribe}
                    >
                        subscribe
                    </button>
                }
            >
                <button class="editor__button" disabled={!managed.unsaved() || session.busy()} onClick={managed.keep}>
                    save
                </button>
                <button
                    class="editor__button editor__button--line"
                    disabled={!managed.unsaved() || session.busy()}
                    onClick={managed.restore}
                >
                    revert
                </button>
                <button
                    class="editor__button editor__button--danger"
                    disabled={session.busy()}
                    onClick={managed.leave}
                >
                    unsubscribe
                </button>
            </Show>
        </div>
    );
}

export function Head() {
    const { session, rules, managed } = useEditor();

    const withheld = () => {
        const open = managed.open();

        return open ? open.offered !== "active" : false;
    };

    const sub = () => {
        const open = managed.open();

        if (open) return open.mode ? open.id : open.id + " · not subscribed";

        const own = rules.open();

        if (!own) return session.view()?.name ?? "";

        return own.id + " · " + own.mode;
    };

    return (
        <div class="rulehead">
            <div>
                <Name />
                <p class={withheld() ? "rulehead__sub rulehead__sub--warned" : "rulehead__sub"}>{sub()}</p>
            </div>

            <div class="rulehead__right">
                <div class="rulehead__row">
                    <Ladder />

                    <Show when={managed.open()} fallback={<Own />}>
                        {(open) => <Shared managed={open()} />}
                    </Show>
                </div>

                <Status />
            </div>
        </div>
    );
}
