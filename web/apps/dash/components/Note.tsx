import { Switch, Match } from "solid-js";
import type { JSX } from "solid-js";

import { signIn } from "../api.ts";
import type { Refusal } from "../api.ts";

export function Note(props: { headline: string; bad?: boolean; children?: JSX.Element }) {
    return (
        <div class={props.bad ? "note note--bad" : "note"}>
            <h2 class="note__title">{props.headline}</h2>
            {props.children}
        </div>
    );
}

export function Explain(props: { error?: Refusal }) {
    return (
        <Switch fallback={<Note headline="unreadable response" bad />}>
            <Match when={props.error === "anonymous"}>
                <Note headline="signed out">
                    <p class="note__body">
                        <a class="dashboard__button" href={signIn()}>
                            sign in with discord
                        </a>
                    </p>
                </Note>
            </Match>

            <Match when={props.error === "forbidden"}>
                <Note headline="missing manage server" bad />
            </Match>

            <Match when={props.error === "absent"}>
                <Note headline="aegis not in this server" bad />
            </Match>

            <Match when={props.error === "unreachable"}>
                <Note headline="bot unreachable" bad>
                    <p class="note__body">no changes saved</p>
                </Note>
            </Match>
        </Switch>
    );
}
