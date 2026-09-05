import { useParams } from "@solidjs/router";
import { Match, Show, Switch } from "solid-js";

import { Frame } from "./components/Chrome.tsx";
import { Clauses } from "./components/Clauses.tsx";
import { Managed } from "./components/Managed.tsx";
import { NoRules } from "./components/NoRules.tsx";
import { Preview } from "./components/Preview.tsx";
import { RuleList } from "./components/RuleList.tsx";
import { Stubs } from "./components/Stubs.tsx";
import { Vocab } from "./components/Vocab.tsx";
import { EditorProvider, useEditor } from "./state/editor.tsx";

function Work() {
    const { rules, managed } = useEditor();

    return (
        <Frame>
            <RuleList />

            <Switch fallback={<NoRules />}>
                <Match when={managed.open()}>
                    <Managed />
                    <Stubs />
                </Match>

                <Match when={rules.open()}>
                    <div class="editor__col">
                        <Clauses />
                        <Preview />
                    </div>

                    <Vocab />
                </Match>
            </Switch>
        </Frame>
    );
}

export default function Editor() {
    const params = useParams();

    return (
        <Show when={{ guild: params.guild ?? "" }} keyed>
            {(params) => (
                <EditorProvider mode="guild" guild={params.guild}>
                    <Work />
                </EditorProvider>
            )}
        </Show>
    );
}
