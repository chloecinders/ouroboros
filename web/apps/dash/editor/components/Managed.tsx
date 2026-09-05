import { Show } from "solid-js";

import { useEditor } from "../state/editor.tsx";
import type { Taken } from "../state/managed.ts";
import { Clauses } from "./Clauses.tsx";

function Unpublished(props: { managed: Taken }) {
    return (
        <Show when={props.managed.mode && props.managed.offered !== "active"}>
            <div class="hint hint--warn">
                set to <code class="hint__code">{props.managed.offered}</code> by the developers
            </div>
        </Show>
    );
}

export function Managed() {
    const { managed } = useEditor();

    return (
        <Show when={managed.open()}>
            {(managed) => (
                <div class="editor__col">
                    <div class="colhead colhead--grey">
                        <span>{managed().name}</span>
                    </div>

                    <div class="hint">{managed().description}</div>

                    <Unpublished managed={managed()} />

                    <Show when={managed().mode}>
                        <Clauses />
                    </Show>
                </div>
            )}
        </Show>
    );
}
