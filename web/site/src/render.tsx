import type { JSX } from "solid-js";
import { renderToString } from "solid-js/web";

import { Layout } from "./Layout.tsx";
import { Home } from "./pages/Home.tsx";
import { Privacy } from "./pages/Privacy.tsx";
import { Terms } from "./pages/Terms.tsx";
import { Command } from "./wiki/Command.tsx";
import { Wiki } from "./wiki/Wiki.tsx";
import type { Sheet } from "./wiki/commands.ts";
import { grouped, safeName, titleCase } from "./wiki/commands.ts";

export interface Prosed {
    name: string;
    title: string;
    html: string;
}

export interface Written {
    title: string;
    description?: string;
    active?: string;
    body: () => JSX.Element;
}

export interface Rendered {
    file: string;
    html: string;
}

export interface Input {
    sheet: Sheet;
    prose: (name: string) => string;
    general: Prosed[];
    updated: string;
}

function address(file: string): string {
    const clean = file.replace(/\.html$/, "");

    return clean === "index" ? "/" : `/${clean}`;
}

function page(file: string, written: Written): Rendered {
    const html =
        "<!doctype html>" +
        renderToString(() => (
            <Layout
                title={written.title}
                description={written.description}
                active={written.active}
                path={address(file)}
            >
                {written.body()}
            </Layout>
        ));

    return { file, html };
}

export function pages({ sheet, prose, general, updated }: Input): Rendered[] {
    const { commands, categories } = grouped(sheet);
    const index = general.map((prosed) => ({ name: prosed.name, title: prosed.title }));

    const out: Rendered[] = [];

    const wiki = (active: string, inner: () => JSX.Element) => () => (
        <Wiki general={index} categories={categories} active={active}>
            {inner()}
        </Wiki>
    );

    out.push(
        page("index.html", {
            title: "Home",
            description: "Aegis is a high-performance Discord moderation bot built with Rust.",
            active: "index",
            body: () => <Home />,
        }),
    );

    out.push(
        page("terms.html", {
            title: "Terms of Service",
            description: "Terms of Service for the Aegis Discord moderation bot.",
            active: "terms",
            body: () => <Terms updated={updated} />,
        }),
    );

    out.push(
        page("privacy.html", {
            title: "Privacy Policy",
            description: "Privacy Policy for the Aegis Discord moderation bot.",
            active: "privacy",
            body: () => <Privacy updated={updated} />,
        }),
    );

    for (const prosed of general) {
        const body = wiki(prosed.name, () => <div innerHTML={prosed.html} />);

        out.push(page(`wiki/${prosed.name}.html`, { title: prosed.title, active: "wiki", body }));

        if (prosed.name === "overview") out.push(page("wiki.html", { title: "Wiki", active: "wiki", body }));
    }

    for (const cmd of commands) {
        out.push(
            page(`wiki/${safeName(cmd.name)}.html`, {
                title: titleCase(cmd.name),
                description: cmd.short || cmd.full,
                active: "wiki",
                body: wiki(cmd.name, () => <Command cmd={cmd} prose={prose(cmd.name)} />),
            }),
        );
    }

    return out;
}
