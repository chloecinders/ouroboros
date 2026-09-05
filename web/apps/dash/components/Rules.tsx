import { A } from "@solidjs/router";
import { For, Show, createResource } from "solid-js";

import type { Offered, Saved } from "../api.ts";
import { API, ask } from "../api.ts";
import { editor, useGuild } from "./Guild.tsx";
import { Explain, Note } from "./Note.tsx";

export function Rules() {
    const guild = useGuild();
    const [guildCtx] = createResource(
        () => guild().id,
        (id) => ask<Saved[]>(API.rules(id)),
    );
    const [shared] = createResource(
        () => guild().id,
        (id) => ask<Offered[]>(API.managed_rules(id)),
    );

    const offers = () => {
        const answer = shared();

        return answer && !answer.error ? answer.value : [];
    };

    const taken = () => offers().filter((one) => one.mode);
    const managedAt = () => editor(guild().id) + "?managed=" + encodeURIComponent(offers()[0].id);

    return (
        <Show when={guildCtx()}>
            {(answer) => (
                <Show when={!answer().error} fallback={<Explain error={answer().error} />}>
                    <Show
                        when={answer().value.length}
                        fallback={
                            <Show
                                when={!taken().length}
                                fallback={
                                    <Note headline="no rules found">
                                        <p class="note__body">
                                            <A class="dashboard__button" href={editor(guild().id)}>
                                                open the editor
                                            </A>
                                        </p>
                                    </Note>
                                }>
                                <Note headline="no rules found">
                                    <p class="note__body">
                                        <A class="dashboard__button" href={editor(guild().id)}>
                                            create a rule
                                        </A>
                                    </p>
                                </Note>
                            </Show>
                        }>
                        <div class="section">
                            <span>rules</span>
                            <Show when={offers().length}>
                                <A class="section__link" href={managedAt()}>
                                    managed rules
                                </A>
                            </Show>
                        </div>

                        <div class="rules">
                            <div class="rules__head">
                                <span>name</span>
                                <span>mode</span>
                            </div>

                            <For each={answer().value}>
                                {(rule) => {
                                    const at = editor(guild().id) + "?rule=" + encodeURIComponent(rule.id);

                                    return (
                                        <A class="rules__row" href={at}>
                                            <span class="rules__name">{rule.name}</span>
                                            <span class={"pill pill--" + rule.mode}>{rule.mode}</span>
                                        </A>
                                    );
                                }}
                            </For>
                        </div>
                    </Show>
                </Show>
            )}
        </Show>
    );
}
