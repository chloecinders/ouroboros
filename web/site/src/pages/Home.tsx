import { For } from "solid-js";

interface Row {
    title: string;
    body: string;
    link?: { at: string; says: string; away?: boolean };
}

export function Home() {
    return (
        <>
            <div class="hero">
                <h1 class="hero__title">Aegis</h1>
                <p class="hero__blurb">An actually modern Discord moderation bot made to be fast and powerful.</p>

                <div class="cta">
                    <a class="cta__link" href="https://discord.gg/SdUf7TrbDq" target="_blank">
                        Join Discord Server
                    </a>
                    <a class="cta__link" href="https://github.com/chloecinders/aegis" target="_blank">
                        See Source
                    </a>
                </div>
            </div>

            <div class="rows">
                <For
                    each={
                        [
                            {
                                title: "Modern Semantics",
                                body: "Infers arguments intuitively using replies. Reply to someone or a log and the bot fills in the rest instantly. Convenient, fast and simple.",
                            },
                            {
                                title: "Advanced Logging",
                                body: "Permanent message retention. View the edit history of messages, see exactly which moderator deleted a message. Optional encryption, you don't want us to see your messages and neither do we.",
                            },
                            {
                                title: "Better Automod",
                                body: "Automod that can even check images and automatically take any action. Keep those scam bots away permanently.",
                            },
                            {
                                title: "Zero Bloat",
                                body: "No leveling, no economy, definitely no NFTs. Aegis will always stay only a moderation bot, never exploding the scope and always focusing on bringing you the best of moderation.",
                            },
                            {
                                title: "Don't trust us?",
                                body: "The entire bot is open source and self hosting is encouraged. Moderation tools should be open, something keeping your server safe must be able to be audited by everyone.",
                                link: { at: "https://github.com/chloecinders/aegis", says: "See Source", away: true },
                            },
                        ] as Row[]
                    }
                >
                    {(row, index) => (
                        <article class="rows__item">
                            <span class="rows__num">{String(index() + 1).padStart(2, "0")}</span>
                            <h3 class="rows__title">{row.title}</h3>
                            <p class="rows__body">{row.body}</p>
                            {row.link ? (
                                <a class="rows__link" href={row.link.at} target={row.link.away ? "_blank" : undefined}>
                                    {row.link.says}
                                </a>
                            ) : null}
                        </article>
                    )}
                </For>
            </div>
        </>
    );
}
