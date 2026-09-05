import { For, Show } from "solid-js";

import { parse } from "../parse.ts";
import { useEditor } from "../state/editor.tsx";

function Own() {
    const { focus, editing, rules, managed } = useEditor();

    async function switchTo(id: string) {
        if (await managed.leaving()) rules.switchTo(id);
    }

    return (
        <For each={rules.list}>
            {(rule) => {
                const open = () => rule.id === focus.rule();
                const parsed = () => parse(open() ? editing.draft() : rule.source);

                return (
                    <a
                        class={open() ? "rulelist__item rulelist__item--on" : "rulelist__item"}
                        onClick={() => switchTo(rule.id)}
                    >
                        <span class="rulelist__name">
                            {rule.name}
                            <Show when={open() && rules.dirty()}> <em class="rulelist__dirty">*</em></Show>
                        </span>
                        <span class="rulelist__meta">
                            <span class={"rulelist__state rulelist__state--" + rule.mode}>{rule.mode}</span>
                            <span class={parsed().errors ? "rulelist__bad" : ""}>
                                {parsed().errors ? parsed().errors + " err" : ""}
                            </span>
                        </span>
                    </a>
                );
            }}
        </For>
    );
}

function Shared() {
    const { focus, managed } = useEditor();

    return (
        <For each={managed.list}>
            {(offer) => {
                const open = () => offer.id === focus.managed();
                const subscription = () => offer.mode || "not subscribed";

                return (
                    <a
                        class={
                            open()
                                ? "rulelist__item rulelist__item--managed rulelist__item--on"
                                : "rulelist__item rulelist__item--managed"
                        }
                        onClick={() => managed.show(offer.id)}
                    >
                        <span class="rulelist__name">
                            {offer.name}
                            <span class="rulelist__lock" title="written by the developers, clauses not shown">
                                managed
                            </span>
                        </span>
                        <span class="rulelist__meta">
                            <span
                                class={
                                    offer.mode
                                        ? "rulelist__state rulelist__state--" + offer.mode
                                        : "rulelist__state"
                                }
                            >
                                {subscription()}
                            </span>
                            <Show when={offer.mode && offer.offered !== "active"}>
                                <span class="rulelist__bad">unpublished</span>
                            </Show>
                        </span>
                    </a>
                );
            }}
        </For>
    );
}

export function RuleList() {
    const { session, rules, managed } = useEditor();

    async function create() {
        if (await managed.leaving()) rules.create();
    }

    return (
        <div class="editor__col">
            <div class="colhead">
                <span>rules</span>
            </div>

            <div class="rulelist">
                <Own />
            </div>

            <button class="new-rule" disabled={session.busy()} onClick={create}>
                + new rule
            </button>

            <Show when={managed.list.length}>
                <div class="colhead colhead--grey">
                    <span>managed</span>
                    <span>{managed.list.filter((offer) => offer.mode).length + "/" + managed.list.length}</span>
                </div>

                <div class="rulelist">
                    <Shared />
                </div>
            </Show>
        </div>
    );
}
