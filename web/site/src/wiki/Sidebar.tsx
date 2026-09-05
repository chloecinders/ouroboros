import { For } from "solid-js";

import type { Grouping } from "./commands.ts";
import { safeName } from "./commands.ts";

export interface Listed {
    name: string;
    title: string;
}

export interface SidebarProps {
    general: Listed[];
    categories: Grouping[];
    active?: string;
}

export function Sidebar(props: SidebarProps) {
    const here = (name: string) =>
        props.active === name ? "sidebar__link sidebar__link--active" : "sidebar__link";

    return (
        <div id="sidebar-commands">
            <h3 class="sidebar__group">General</h3>
            <ul class="sidebar__list">
                <For each={props.general}>
                    {(page) => (
                        <li>
                            <a href={"/wiki/" + page.name} class={here(page.name)}>
                                {page.title}
                            </a>
                        </li>
                    )}
                </For>
            </ul>

            <For each={props.categories}>
                {(category) => (
                    <>
                        <h3 class="sidebar__group">{category.name}</h3>
                        <ul class="sidebar__list">
                            <For each={category.commands}>
                                {(cmd) => (
                                    <li>
                                        <a href={"/wiki/" + safeName(cmd.name)} class={here(cmd.name)}>
                                            {cmd.name}
                                        </a>
                                    </li>
                                )}
                            </For>
                        </ul>
                    </>
                )}
            </For>
        </div>
    );
}
