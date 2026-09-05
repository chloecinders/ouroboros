import { A } from "@solidjs/router";
import type { JSX } from "solid-js";
import { Show } from "solid-js";

import { Footer, Top } from "../../../../shared/chrome.tsx";
import { ACTIVITY } from "../../api.ts";
import { useEditor } from "../state/editor.tsx";
import { Dialog } from "./Dialog.tsx";
import { Head } from "./Head.tsx";
import { RefusalNotice } from "./RefusalNotice.tsx";

export function Navbar() {
    const { session, rules, managed } = useEditor();

    const back = () => {
        const view = session.view();

        return view && !session.authored ? "/dashboard/" + encodeURIComponent(view.id) : "/dashboard";
    };
    const open = () => managed.open() || rules.open();
    const now = () => open()?.name ?? "none";

    const crumb = (
        <span class="crumb">
            <Show when={!ACTIVITY}>
                <A href="/dashboard">servers</A>
                <span>/</span>
            </Show>

            <A href={back()}>{session.view()?.name ?? "server"}</A>
            <span>/</span>
            <span class="crumb__now">{now()}</span>
        </span>
    );

    return (
        <Top embedded={ACTIVITY} link={A} crumb={crumb}>
            <Show when={!ACTIVITY}>
                <a href="/logout">sign out</a>
            </Show>
        </Top>
    );
}

export function Frame(props: { children: JSX.Element }) {
    const { session, rules, managed } = useEditor();

    const open = () => managed.open() || rules.open();

    return (
        <div class="editor">
            <div class="slab" />

            <div class="editor__app">
                <Navbar />

                <Show
                    when={!session.refused()}
                    fallback={<RefusalNotice error={session.refused()} kept={session.authored} />}
                >
                    <Show when={session.ready()} fallback={<div class="editor__loading">loading</div>}>
                        <Show when={open()}>
                            <Head />
                        </Show>

                        <div class="editor__work">{props.children}</div>
                    </Show>
                </Show>

                <Footer class="editor__footer" stamp={ACTIVITY ? "in discord" : location.host} />

                <Dialog />
            </div>
        </div>
    );
}
