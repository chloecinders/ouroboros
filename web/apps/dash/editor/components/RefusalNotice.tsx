import { Match, Switch } from "solid-js";

import type { Refusal } from "../../api.ts";

export function RefusalNotice(props: { error: Refusal | null; kept?: boolean }) {
    const signIn = () => "/login?next=" + encodeURIComponent(location.pathname + location.search);

    return (
        <Switch
            fallback={
                <div class="notice notice--bad">
                    <h2 class="notice__title">bot unreachable</h2>
                    <p class="notice__body">no changes saved</p>
                </div>
            }>
            <Match when={props.error === "anonymous"}>
                <div class="notice">
                    <h2 class="notice__title">signed out</h2>
                    <p class="notice__body">
                        <a class="editor__button" href={signIn()}>
                            sign in with discord
                        </a>
                    </p>
                </div>
            </Match>

            <Match when={props.error === "forbidden" && props.kept}>
                <div class="notice notice--bad">
                    <h2 class="notice__title">missing access</h2>
                    <p class="notice__body">managed rules can only be written by bot developers</p>
                </div>
            </Match>

            <Match when={props.error === "forbidden"}>
                <div class="notice notice--bad">
                    <h2 class="notice__title">missing manage server</h2>
                </div>
            </Match>

            <Match when={props.error === "absent"}>
                <div class="notice notice--bad">
                    <h2 class="notice__title">aegis not in this server</h2>
                </div>
            </Match>
        </Switch>
    );
}
