import { For, Show } from "solid-js";

import { useEditor } from "../state/editor.tsx";

export function Diags() {
    const { session, editing, rules, managed } = useEditor();

    const whole = () => editing.reading().diags.filter((d) => !d.line);
    const inline = () => editing.reading().diags.length - whole().length;

    return (
        <div class="diags">
            <Show when={managed.open() || rules.open()} fallback={<div class="diags__clean">no rule open</div>}>
                <Show
                    when={whole().length}
                    fallback={
                        <div class="diags__clean">
                            {inline()
                                ? inline() + " problem" + (inline() > 1 ? "s" : "") + " marked above"
                                : "no problems"}
                        </div>
                    }
                >
                    <For each={whole()}>
                        {(d) => (
                            <button
                                class={
                                    d.level === "error"
                                        ? "diags__item diags__item--error"
                                        : "diags__item diags__item--warn"
                                }
                                onClick={() => session.surface.source?.focus()}
                            >
                                <span class="diags__line">rule</span>
                                <span class="diags__mark">{d.level === "error" ? "x" : "!"}</span>
                                <span>{d.msg}</span>
                            </button>
                        )}
                    </For>
                </Show>
            </Show>
        </div>
    );
}
