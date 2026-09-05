import { For, Show } from "solid-js";

import type { Documented, Flag, Token } from "./commands.ts";
import { highlightExample, highlightSyntax, titleCase } from "./commands.ts";

function Highlighted(props: { tokens: Token[] }) {
    return (
        <code class="code-box__code">
            <For each={props.tokens}>
                {(token) => <span class={"token token--" + token.kind}>{token.text}</span>}
            </For>
        </code>
    );
}

function Badges(props: { label: string; names: string[] }) {
    return (
        <Show when={props.names.length}>
            <span class="command-meta__row">
                <strong class="command-meta__label">{props.label}:</strong>{" "}
                <For each={props.names}>
                    {(name, index) => (
                        <>
                            {index() ? " " : ""}
                            <span class="badge badge--permission">{name}</span>
                        </>
                    )}
                </For>
            </span>
        </Show>
    );
}

function Parameters(props: { flags: Flag[] }) {
    return (
        <Show when={props.flags.length}>
            <h2>Parameters</h2>
            <table class="params-table">
                <thead>
                    <tr>
                        <th class="params-table__head">Name</th>
                        <th class="params-table__head">Flag</th>
                        <th class="params-table__head">Description</th>
                    </tr>
                </thead>
                <tbody>
                    <For each={props.flags}>
                        {(flag) => (
                            <tr>
                                <td class="params-table__cell">
                                    <code>{flag.name}</code>
                                </td>
                                <td class="params-table__cell">
                                    <code>{flag.switch}</code>
                                </td>
                                <td class="params-table__cell" innerHTML={flag.desc} />
                            </tr>
                        )}
                    </For>
                </tbody>
            </table>
        </Show>
    );
}

export function Command(props: { cmd: Documented; prose: string }) {
    const cmd = () => props.cmd;
    const invocation = (rest: string) => (rest ? "+" + cmd().name + " " + rest : "+" + cmd().name);

    return (
        <>
            <h1>{titleCase(cmd().name)}</h1>
            <p innerHTML={cmd().full || cmd().short} />

            <div class="command-meta">
                <span class="command-meta__row">
                    <strong class="command-meta__label">Category:</strong> <span class="badge badge--category">{cmd().category}</span>
                </span>

                <Show when={cmd().aliases.length}>
                    <span class="command-meta__row">
                        <strong class="command-meta__label">Aliases:</strong>{" "}
                        <For each={cmd().aliases}>
                            {(alias, index) => (
                                <>
                                    {index() ? " " : ""}
                                    <code>+{alias}</code>
                                </>
                            )}
                        </For>
                    </span>
                </Show>

                <Badges label="Required Permissions" names={cmd().user} />
                <Badges label="One of these Permissions" names={cmd().one_of} />
            </div>

            <h2>Usage</h2>
            <div class="code-box">
                <Highlighted tokens={highlightSyntax(invocation(cmd().syntax))} />
            </div>

            <h2>Examples</h2>
            <div class="code-box code-box--examples">
                <Highlighted tokens={highlightExample(invocation(cmd().example))} />
            </div>

            <Parameters flags={cmd().flags} />

            <Show when={props.prose}>
                <div class="wiki-content__extra" style="margin-top: 40px" innerHTML={props.prose} />
            </Show>
        </>
    );
}
