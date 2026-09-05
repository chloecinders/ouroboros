import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { createStore } from "solid-js/store";
import { render } from "solid-js/web";

import { Account, Footer, Top } from "../../shared/chrome.tsx";
import type { Answer, Header, Rendered, Viewer } from "./api.ts";
import { Refused, messages, meta, viewer } from "./api.ts";
import { Message } from "./components/Message.tsx";

interface Note {
    title: string;
    body?: string;
    back?: string;
}

interface Line {
    message: Rendered;
    grouped: boolean;
}

interface Block {
    at: number;
    lines: Line[];
}

function refusal(status: number): Note {
    if (status === 401)
        return {
            title: "Sign in required",
            body: "You do not have the required permissions to read this transcript.",
            back: "/login?next=" + encodeURIComponent(location.pathname),
        };

    if (status === 403)
        return {
            title: "Not permitted",
            body: "You do not have the required permissions to read this transcript.",
        };

    return { title: "Transcript not found" };
}

function Head(props: { meta: Header | null; viewer: Viewer | null; fetched: boolean }) {
    const meta = () => props.meta;

    return (
        <header class="head">
            <div class="page">
                <Top crumb={<span class="head__kind">Transcript</span>}>
                    <Show when={props.viewer}>{(account) => <Account viewer={account()} />}</Show>
                </Top>

                <div class="band">
                    <h1 class="band__title">{meta()?.title || "Transcript"}</h1>

                    <div class="static">
                        <Show
                            when={meta()}
                            fallback={
                                <Show when={!props.fetched}>
                                    <span class="static__item">Loading</span>
                                </Show>
                            }
                        >
                            {(head) => (
                                <span class="static__item">
                                    Saved <b class="static__value">{new Date(head().created_at).toLocaleString()}</b>
                                </span>
                            )}
                        </Show>
                    </div>
                </div>
            </div>
        </header>
    );
}

function Said(props: { note: Note }) {
    return (
        <div class="note">
            <h2 class="note__title">{props.note.title}</h2>
            <Show when={props.note.body}>
                <p class="note__body">{props.note.body}</p>
            </Show>

            <Show when={props.note.back}>
                {(back) => (
                    <a class="note__link" href={back()}>
                        Sign in with Discord
                    </a>
                )}
            </Show>
        </div>
    );
}

function Split(props: { name: string }) {
    return (
        <div class="split">
            <span>#{props.name}</span>
        </div>
    );
}

function Run(props: {
    block: Block;
    before?: Rendered;
    names: Map<string, string>;
    spansChannels?: boolean;
    jumpable?: boolean;
    measured: (node: HTMLDivElement | null) => void;
}) {
    let node: HTMLDivElement | undefined;

    onMount(() => props.measured(node ?? null));
    onCleanup(() => props.measured(null));

    const before = (at: number) => (at === 0 ? props.before : props.block.lines[at - 1].message);

    return (
        <div ref={node}>
            <For each={props.block.lines}>
                {(line, at) => [
                    <Show when={!!props.spansChannels && before(at())?.channel !== line.message.channel}>
                        <Split name={props.names.get(line.message.channel) || line.message.channel} />
                    </Show>,
                    <Message message={line.message} grouped={line.grouped} jumpable={props.jumpable} />,
                ]}
            </For>
        </div>
    );
}

function Transcript() {
    const [head, setHead] = createSignal<Header | null>(null);
    const [account, setAccount] = createSignal<Viewer | null>(null);
    const [note, setNote] = createSignal<Note | null>(null);
    const [fetched, setFetched] = createSignal(false);
    const [blocks, setBlocks] = createSignal<Block[]>([]);
    const [ended, setEnded] = createSignal(false);
    const [loading, setLoading] = createSignal(false);
    const [heights, setHeights] = createStore<number[]>([]);

    const nodes: (HTMLDivElement | null)[] = [];
    let after: string | null = null;
    let anchor: Rendered | null = null;

    const names = createMemo(() => new Map((head()?.channels || []).map((channel) => [channel.id, channel.name])));

    const from = () => Math.max(0, blocks().length - 4);
    const live = () => blocks().slice(from());
    const spacer = () => heights.slice(0, from()).reduce((total, height) => total + (height || 0), 0);

    function measure() {
        for (const [at, node] of nodes.entries()) if (node) setHeights(at, node.offsetHeight);
    }

    async function more() {
        if (loading() || ended()) return;

        setLoading(true);

        let page: Answer;

        try {
            page = await messages(after);
        } catch {
            setNote({ title: "Load failed" });
            setEnded(true);
            setLoading(false);

            return;
        }

        measure();

        const next = [...blocks()];
        const lines: Line[] = [];

        for (const message of page.messages) {
            const grouped =
                anchor !== null &&
                anchor.author === message.author &&
                anchor.channel === message.channel &&
                new Date(message.at).getTime() - new Date(anchor.at).getTime() < 300000;

            if (!grouped) anchor = message;

            lines.push({ message, grouped });
        }

        for (let at = 0; at < lines.length; at += 100)
            next.push({ at: next.length, lines: lines.slice(at, at + 100) });

        setBlocks(next);

        after = page.next;

        setEnded(page.next === null || page.next === undefined);
        setLoading(false);

        if (!ended() && document.body.scrollHeight <= innerHeight) more();
    }

    onMount(async () => {
        viewer()
            .then(setAccount)
            .catch(() => setAccount(null));

        let found: Header;

        try {
            found = await meta();
        } catch (failure) {
            setNote(refusal(failure instanceof Refused ? failure.status : 0));
            setFetched(true);

            return;
        }

        setHead(found);
        setFetched(true);

        more();
    });

    const scrolled = () => {
        if (innerHeight + scrollY >= document.body.offsetHeight - 800) more();
    };

    addEventListener("scroll", scrolled, { passive: true });
    onCleanup(() => removeEventListener("scroll", scrolled));

    createEffect(() => {
        const title = head()?.title;

        if (title) document.title = "Transcript - " + title;
    });

    return (
        <div class="transcript">
            <div class="slab" />

            <Head meta={head()} viewer={account()} fetched={fetched()} />

            <main class="page">
                <div aria-hidden="true" style={{ height: `${spacer()}px` }} />

                <div class="transcript__blocks">
                    <For each={live()}>
                        {(block) => (
                            <Run
                                block={block}
                                before={blocks()[block.at - 1]?.lines.at(-1)?.message}
                                names={names()}
                                spansChannels={head()?.spans_channels}
                                jumpable={head()?.jumpable}
                                measured={(node) => (nodes[block.at] = node)}
                            />
                        )}
                    </For>
                </div>

                <Show when={loading()}>
                    <div class="status">Loading</div>
                </Show>

                <Show when={note()}>{(said) => <Said note={said()} />}</Show>
            </main>

            <Footer class="page transcript__footer" />
        </div>
    );
}

const root = document.getElementById("app");

if (!root) throw new Error("no #app found");

render(Transcript, root);
