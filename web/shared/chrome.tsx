import type { Component, JSX } from "solid-js";
import { For, Show } from "solid-js";
import { Dynamic } from "solid-js/web";

interface Linked {
    href: string;
    label: string;
    name?: string;
    app?: boolean;
}

export interface TopProps {
    active?: string;
    link?: Component<{ href: string; class?: string; children: JSX.Element }>;
    embedded?: boolean;
    crumb?: JSX.Element;
    children?: JSX.Element;
}

export function Top(props: TopProps) {
    const on = (link: Linked) =>
        props.active && link.name === props.active ? "top__link top__link--active" : "top__link";

    return (
        <div class="top">
            <Show when={!props.embedded} fallback={<span class="top__mark">Aegis</span>}>
                <a href="/" class="top__mark">
                    Aegis
                </a>
            </Show>

            {props.crumb}

            <nav class="top__nav">
                <Show when={!props.embedded}>
                    <For
                        each={
                            [
                                { href: "/dashboard", label: "Dashboard", app: true },
                                { href: "/wiki/overview", label: "Wiki", name: "wiki" },
                                { href: "/terms", label: "Terms", name: "terms" },
                                { href: "/privacy", label: "Privacy", name: "privacy" },
                                { href: "https://github.com/chloecinders/aegis", label: "GitHub" },
                                { href: "https://discord.gg/SdUf7TrbDq", label: "Discord" },
                            ] as Linked[]
                        }
                    >
                        {(link) => (
                            <Dynamic
                                component={link.app && props.link ? props.link : "a"}
                                href={link.href}
                                class={on(link)}
                            >
                                {link.label}
                            </Dynamic>
                        )}
                    </For>
                </Show>

                {props.children}
            </nav>
        </div>
    );
}

export interface Viewer {
    name: string;
    display: string | null;
    avatar: string | null;
}

export function Account(props: { viewer: Viewer }) {
    return (
        <span class="account">
            <Show when={props.viewer.avatar}>
                <img class="account__avatar" src={props.viewer.avatar ?? undefined} alt="" />
            </Show>
            <span>{props.viewer.display || props.viewer.name}</span>
            <a class="account__signout" href="/logout">
                sign out
            </a>
        </span>
    );
}

export function Footer(props: { stamp?: string; class?: string }) {
    return (
        <footer class={props.class}>
            <span>&copy; 2026 Chloe Cinders &amp; Contributors</span>

            <Show when={props.stamp}>{(stamp) => <span>{stamp()}</span>}</Show>
        </footer>
    );
}
