import type { JSX } from "solid-js";

import type { Listed } from "./Sidebar.tsx";
import { Sidebar } from "./Sidebar.tsx";
import type { Grouping } from "./commands.ts";

export interface WikiProps {
    general: Listed[];
    categories: Grouping[];
    active?: string;
    children?: JSX.Element;
}

export function Wiki(props: WikiProps) {
    return (
        <div class="wiki-layout">
            <aside class="sidebar">
                <Sidebar general={props.general} categories={props.categories} active={props.active} />
            </aside>

            <article class="wiki-content">{props.children}</article>
        </div>
    );
}
