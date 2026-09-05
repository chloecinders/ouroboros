import { For, Show, createResource } from "solid-js";

import type { Trouble } from "../api.ts";
import { API, ask } from "../api.ts";
import { useGuild } from "./Guild.tsx";
import { Explain, Note } from "./Note.tsx";

const time = (at: string) =>
    new Date(at).toLocaleString(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
    });

function Row(props: { trouble: Trouble }) {
    return (
        <div class="faults__row">
            <div class="faults__what">
                <p class="faults__text">{props.trouble.headline}</p>
                <Show when={props.trouble.detail}>
                    <pre class="faults__detail">{props.trouble.detail}</pre>
                </Show>
            </div>

            <span class="faults__at">{time(props.trouble.at)}</span>
        </div>
    );
}

export function Errors() {
    const guild = useGuild();
    const [errors] = createResource(
        () => guild().id,
        (id) => ask<Trouble[]>(API.errors(id)),
    );

    return (
        <Show when={errors()}>
            {(answer) => (
                <Show when={!answer().error} fallback={<Explain error={answer().error} />}>
                    <div class="section">
                        <span>errors</span>
                    </div>

                    <Show when={answer().value.length} fallback={<Note headline="no errors found" />}>
                        <div class="faults">
                            <div class="faults__head">
                                <span>error</span>
                                <span>time</span>
                            </div>

                            <For each={answer().value}>{(row) => <Row trouble={row} />}</For>
                        </div>
                    </Show>
                </Show>
            )}
        </Show>
    );
}
