import { A } from "@solidjs/router";
import { For, Show } from "solid-js";

import { Crest } from "./Chrome.tsx";
import { Note } from "./Note.tsx";
import { useSession } from "./Shell.tsx";

export function Servers() {
    const bot = useSession();
    const listed = () => bot()?.manages ?? [];

    return (
        <Show when={bot()}>
            {(user) => (
                <>
                    <div class="band">
                        <Crest name={user().display || user().name} icon={user().avatar} />
                        <div>
                            <h1 class="band__title">servers</h1>
                        </div>

                        <Show when={user().developer}>
                            <A class="dashboard__button" href="/dashboard/managed_rules">
                                managed rules
                            </A>
                        </Show>
                    </div>

                    <Show
                        when={listed().length}
                        fallback={
                            <Note headline="no servers found">
                                <p class="note__body">aegis is not in a server you manage</p>
                                <p class="note__body">
                                    manage server is read once at sign in, sign out and back in after a role change
                                </p>
                            </Note>
                        }
                    >
                        <div class="section">
                            <span>servers</span>
                        </div>

                        <div class="servers">
                            <For each={listed()}>
                                {(one) => (
                                    <A class="servers__item" href={"/dashboard/" + encodeURIComponent(one.id)}>
                                        <Crest name={one.name} icon={one.icon} />
                                        <span>
                                            <span class="servers__name">{one.name}</span>
                                            <br />
                                            <span class="servers__id">{one.id}</span>
                                        </span>
                                    </A>
                                )}
                            </For>
                        </div>
                    </Show>
                </>
            )}
        </Show>
    );
}
