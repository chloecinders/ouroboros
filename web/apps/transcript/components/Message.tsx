import { For, Match, Show, Switch, createSignal } from "solid-js";

import type { Rendered } from "../api.ts";
import { GUILD } from "../api.ts";

function Attachment(props: { url: string }) {
    const [broken, setBroken] = createSignal(false);
    const image = () => /\.(png|jpe?g|gif|webp|bmp)(\?|$)/i.test(props.url);

    return (
        <Switch>
            <Match when={broken()}>
                <span class="message__lost">image unavailable</span>
            </Match>

            <Match when={image()}>
                <a class="message__file message__file--shot" href={props.url} rel="noreferrer">
                    <img
                        class="message__image"
                        loading="lazy"
                        src={props.url}
                        alt=""
                        onError={() => setBroken(true)}
                    />
                </a>
            </Match>

            <Match when={!image()}>
                <a class="message__file" href={props.url} rel="noreferrer">
                    attachment
                </a>
            </Match>
        </Switch>
    );
}

export function Message(props: { message: Rendered; grouped?: boolean; jumpable?: boolean }) {
    const message = () => props.message;
    const gone = () => message().removed;

    const files = () => message().files || [];
    const stamp = () =>
        new Date(message().at).toLocaleTimeString(undefined, {
            hour: "2-digit",
            minute: "2-digit",
            hour12: false,
        });
    const jump = () => `https://discord.com/channels/${GUILD}/${message().channel}/${message().id}`;
    const [copied, setCopied] = createSignal(false);

    const copy = async (event: MouseEvent) => {
        if (!navigator.clipboard) return;

        event.preventDefault();

        try {
            await navigator.clipboard.writeText(jump());
        } catch (failure) {
            return;
        }

        setCopied(true);
        setTimeout(() => setCopied(false), 1400);
    };

    return (
        <div class={gone() ? "message message--gone" : "message"} id={`m${message().id}`}>
            <Show when={!props.grouped} fallback={<span class="message__stamp">{stamp()}</span>}>
                <Show when={message().avatar} fallback={<div class="message__avatar" />}>
                    <img class="message__avatar" loading="lazy" src={message().avatar ?? undefined} alt="" />
                </Show>
            </Show>

            <div class="message__body">
                <Show when={!props.grouped}>
                    <div>
                        <span class="message__author">{message().display || message().name}</span>
                        <span class="message__time">{new Date(message().at).toLocaleString()}</span>

                        <Show when={message().system}>
                            <span class="message__tag">system</span>
                        </Show>
                    </div>
                </Show>

                <Show when={message().reply_to}>
                    <div class="message__reply">replied</div>
                </Show>

                <div class="message__text">{message().content}</div>

                <Show when={files().length > 0}>
                    <div class="message__files">
                        <For each={files()}>{(url) => <Attachment url={url} />}</For>
                    </div>
                </Show>
            </div>

            <Show when={!gone() && props.jumpable}>
                <a
                    class="message__copy"
                    href={jump()}
                    rel="noreferrer"
                    onClick={copy}
                >
                    {copied() ? "copied" : "copy link"}
                </a>
            </Show>
        </div>
    );
}
