import type { JSX } from "solid-js";

import { Footer, Top } from "../../shared/chrome.tsx";

export const SITE = "https://aegis.chloecinders.com";
const NAME = "Aegis";
const MARK = "/favicon.webp";

export interface LayoutProps {
    title: string;
    description?: string;
    active?: string;
    path?: string;
    children?: JSX.Element;
}

export function Layout(props: LayoutProps) {
    const title = () => `${props.title} - ${NAME}`;
    const says = () => props.description || "An actually modern Discord moderation bot made to be fast and powerful.";
    const here = () => SITE + (props.path || "/");

    return (
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="description" content={says()} />
                <title>{title()}</title>

                <link rel="icon" type="image/webp" href={MARK} />
                <link rel="apple-touch-icon" href={MARK} />
                <link rel="canonical" href={here()} />
                <link rel="stylesheet" href="/styles.css" />

                <meta property="og:type" content="website" />
                <meta property="og:site_name" content={NAME} />
                <meta property="og:title" content={props.title} />
                <meta property="og:description" content={says()} />
                <meta property="og:url" content={here()} />
                <meta property="og:image" content={SITE + MARK} />
                <meta property="og:image:type" content="image/webp" />
                <meta property="og:image:width" content="64" />
                <meta property="og:image:height" content="64" />
                <meta property="og:image:alt" content={`The ${NAME} mark`} />
                <meta property="og:locale" content="en_US" />

                <meta name="twitter:card" content="summary" />
                <meta name="twitter:title" content={props.title} />
                <meta name="twitter:description" content={says()} />
                <meta name="twitter:image" content={SITE + MARK} />

                <meta name="theme-color" content="#04d9b2" />
            </head>

            <body class={props.active === "wiki" ? "wiki-page" : undefined}>
                <div class="slab" />

                <div class="page">
                    <Top active={props.active} />

                    <main>{props.children}</main>

                    <Footer />
                </div>
            </body>
        </html>
    );
}
