import { Show, createEffect, onCleanup } from "solid-js";

import { answer, question } from "../ask.ts";

export function Dialog() {
    const key = (e: KeyboardEvent) => {
        if (!question()) return;

        if (e.key === "Escape") answer(false);
        if (e.key === "Enter") answer(true);
    };

    createEffect(() => {
        if (!question()) return;

        addEventListener("keydown", key);
        onCleanup(() => removeEventListener("keydown", key));
    });

    return (
        <Show when={question()}>
            {(asking) => (
                <div class="scrim" onClick={() => answer(false)}>
                    <div class="dialog" onClick={(e) => e.stopPropagation()}>
                        <h2 class="dialog__title">{asking().headline}</h2>

                        <Show when={asking().body}>
                            <p class="dialog__body">{asking().body}</p>
                        </Show>

                        <div class="dialog__acts">
                            <button class="editor__button editor__button--line" onClick={() => answer(false)}>
                                cancel
                            </button>
                            <button
                                class={asking().danger ? "editor__button editor__button--danger" : "editor__button"}
                                ref={(box) => queueMicrotask(() => box.focus())}
                                onClick={() => answer(true)}>
                                {asking().confirm}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </Show>
    );
}
