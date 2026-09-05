import { A, useNavigate } from "@solidjs/router";
import type { Accessor, JSX } from "solid-js";
import { Match, Show, Switch, createContext, createEffect, createResource, useContext } from "solid-js";

import { Account, Footer, Top } from "../../../shared/chrome.tsx";
import type { Refusal, User } from "../api.ts";
import { ACTIVITY, API, STAMP, ask } from "../api.ts";
import { handshake } from "../discord.ts";
import { Explain, Note } from "./Note.tsx";
import { SignIn } from "./SignIn.tsx";

interface Startup {
    at: "in" | "signin" | "said" | "wrong";
    headline?: string;
    body?: string;
    guild?: string | null;
    viewer?: User;
    error?: Refusal;
}

const Session = createContext<Accessor<User | undefined>>();

export function useSession(): Accessor<User | undefined> {
    const session = useContext(Session);

    if (!session) throw new Error("the session is only readable inside the dashboard shell");

    return session;
}

async function begin(): Promise<Startup> {
    if (ACTIVITY) {
        document.body.classList.add("app", "app--embedded");

        if (!STAMP.client)
            return {
                at: "said",
                headline: "no application configured",
                body: "discord_client_id is not set",
            };

        let guild;

        try {
            guild = await handshake();
        } catch {
            return {
                at: "said",
                headline: "discord sign in failed",
                body: "close this and open it again",
            };
        }

        if (!guild)
            return {
                at: "said",
                headline: "not launched from a server",
                body: "open this from the server to configure",
            };

        return { at: "in", guild };
    }

    const viewer = await ask<User>(API.identity);

    if (viewer.error === "anonymous") return { at: "signin" };
    if (viewer.error) return { at: "wrong", error: viewer.error };

    return { at: "in", viewer: viewer.value };
}

export function Shell(props: { children?: JSX.Element }) {
    const [session] = createResource(begin);
    const navigate = useNavigate();

    const at = (name: Startup["at"]) => session()?.at === name;

    createEffect(() => {
        const guild = session()?.guild;

        if (guild) navigate("/dashboard/" + guild, { replace: true });
    });

    return (
        <Session.Provider value={() => session()?.viewer}>
            <div class="dashboard">
                <div class="slab" />

                <div class="page">
                    <Top embedded={ACTIVITY} link={A}>
                        <Show when={session()?.viewer}>{(viewer) => <Account viewer={viewer()} />}</Show>
                    </Top>

                    <main>
                        <Switch>
                            <Match when={at("signin")}>
                                <SignIn />
                            </Match>

                            <Match when={at("wrong")}>
                                <Explain error={session()?.error} />
                            </Match>

                            <Match when={at("said")}>
                                <Note headline={session()?.headline ?? ""} bad>
                                    <p class="note__body">{session()?.body}</p>
                                </Note>
                            </Match>

                            <Match when={at("in")}>{props.children}</Match>
                        </Switch>
                    </main>

                    <Footer class="dashboard__footer" stamp={ACTIVITY ? "in discord" : location.host} />
                </div>
            </div>
        </Session.Provider>
    );
}
