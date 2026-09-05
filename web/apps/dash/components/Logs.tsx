import { For, Show, createResource, createSignal } from "solid-js";

import type { Definition, Entry } from "../api.ts";
import { API, ask, put, wording } from "../api.ts";
import { useGuild } from "./Guild.tsx";
import { Explain, Note } from "./Note.tsx";

interface Said {
    text: string;
    kind: string;
}

function Picker(props: {
    channels: Entry[];
    chosen: string;
    busy: boolean;
    onPick: (wanted: string) => void;
}) {
    const missing = () =>
        props.chosen && !props.channels.some((channel) => channel.id === props.chosen) ? props.chosen : null;

    return (
        <select
            class="logs__select"
            disabled={props.busy}
            value={props.chosen || ""}
            onChange={(e) => props.onPick(e.currentTarget.value)}
        >
            <option value="">not logged</option>

            <Show when={missing()}>
                {(id) => <option value={id()}>unknown channel {id()}</option>}
            </Show>

            <For each={props.channels}>{(channel) => <option value={channel.id}>#{channel.name}</option>}</For>
        </select>
    );
}

function Row(props: { kind: Definition; guild: string; channels: Entry[] }) {
    const [chosen, setChosen] = createSignal(props.kind.channel || "");
    const [busy, setBusy] = createSignal(false);
    const [said, setSaid] = createSignal<Said | null>(null);

    async function pick(wanted: string) {
        const last = chosen();

        setChosen(wanted);
        setBusy(true);
        setSaid({ text: "saving", kind: "" });

        const answer = await put<null>(API.logs(props.guild), {
            kind: props.kind.kind,
            channel: wanted || null,
        });

        setBusy(false);

        if (answer.error) {
            setChosen(last);
            setSaid({ text: answer.detail ? answer.detail.problem : wording(answer.error), kind: "bad" });
            return;
        }

        setSaid({ text: wanted ? "routed" : "off", kind: "ok" });
    }

    return (
        <div class="logs__row">
            <span class="logs__kind">{props.kind.title}</span>
            <span class="logs__about">{props.kind.about}</span>
            <Picker channels={props.channels} chosen={chosen()} busy={busy()} onPick={pick} />
            <span class={said() ? "logs__said logs__said--" + said()?.kind : "logs__said"}>{said() ? said()?.text : ""}</span>
        </div>
    );
}

export function Logs() {
    const guild = useGuild();
    const [defined] = createResource(
        () => guild().id,
        (id) => ask<Definition[]>(API.logs(id)),
    );

    return (
        <Show when={defined()}>
            {(answer) => (
                <Show when={!answer().error} fallback={<Explain error={answer().error} />}>
                    <div class="section">
                        <span>logs</span>
                    </div>

                    <Show when={guild().channels.length} fallback={<Note headline="no visible channels" bad />}>
                        <div class="logs">
                            <For each={answer().value}>
                                {(kind) => <Row kind={kind} guild={guild().id} channels={guild().channels} />}
                            </For>
                        </div>
                    </Show>
                </Show>
            )}
        </Show>
    );
}
