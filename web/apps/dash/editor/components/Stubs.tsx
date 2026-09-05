import { For, Show } from "solid-js";

import { RESPONSE } from "../grammar.ts";
import { useEditor } from "../state/editor.tsx";

export function Stubs() {
    const { session, editing, managed } = useEditor();

    function append(stub: string) {
        const draft = editing.draft();

        editing.setDraft(draft + (!draft || draft.endsWith("\n") ? "" : "\n") + stub);
        session.say(null);
        editing.check();
    }

    return (
        <div class="editor__col">
            <Show
                when={managed.open()?.mode}
                fallback={
                    <>
                        <div class="colhead colhead--grey">
                            <span>not subscribed</span>
                        </div>

                        <div class="hint">subscribe to set a response</div>
                    </>
                }>
                <div class="colhead colhead--grey">
                    <span>response</span>
                </div>

                <div class="vocab">
                    <For each={RESPONSE}>
                        {(clause) => (
                            <button class="vocab__item" onClick={() => append(clause.stub)}>
                                <span class="vocab__key">{clause.keyword}</span>
                                <span class="vocab__takes">{clause.takes}</span>
                            </button>
                        )}
                    </For>
                </div>
            </Show>
        </div>
    );
}
