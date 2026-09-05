import { For, Show } from "solid-js";

import { CLAUSES } from "../grammar.ts";
import { useEditor } from "../state/editor.tsx";

export function Vocab() {
    const { editing } = useEditor();

    const clause = () => CLAUSES.find((c) => c.keyword === editing.vocab());

    return (
        <div class="editor__col">
            <Show
                when={clause()}
                fallback={
                    <>
                        <div class="colhead">
                            <span>clauses</span>
                        </div>

                        <div>
                            <div class="vocab">
                                <For each={CLAUSES}>
                                    {(c) => (
                                        <button class="vocab__item" onClick={() => editing.pick(c.keyword)}>
                                            <span class="vocab__key">{c.keyword}</span>
                                            <span class="vocab__takes">{c.takes}</span>
                                        </button>
                                    )}
                                </For>
                            </div>
                        </div>
                    </>
                }
            >
                {(picked) => (
                    <>
                        <div class="colhead colhead--teal">
                            <span>{picked().keyword}</span>
                        </div>

                        <div>
                            <button class="back" onClick={editing.unpick}>
                                all clauses
                            </button>

                            <div class="hint">{picked().about}</div>

                            <Show
                                when={picked().values.length}
                                fallback={
                                    <div class="hint" style="border-top:1px solid var(--rule)">
                                        takes nothing
                                    </div>
                                }
                            >
                                <div class="vocab vocab--values">
                                    <For each={picked().values}>
                                        {([value, about]) => (
                                            <button class="vocab__item">
                                                <span class="vocab__key">{value}</span>
                                                <span class="vocab__takes">{about}</span>
                                            </button>
                                        )}
                                    </For>
                                </div>
                            </Show>
                        </div>
                    </>
                )}
            </Show>
        </div>
    );
}
