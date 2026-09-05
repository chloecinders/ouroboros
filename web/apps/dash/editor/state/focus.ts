import type { Accessor } from "solid-js";
import { batch, createSignal } from "solid-js";

export interface Focus {
    rule: Accessor<string | null>;
    managed: Accessor<string | null>;
    onRule: (id: string | null) => void;
    onManaged: (id: string) => void;
}

export function createFocus(): Focus {
    const [rule, setRule] = createSignal<string | null>(null);
    const [managed, setManaged] = createSignal<string | null>(null);

    return {
        rule,
        managed,
        onRule: (id) =>
            batch(() => {
                setManaged(null);
                setRule(id);
            }),
        onManaged: (id) =>
            batch(() => {
                setRule(null);
                setManaged(id);
            }),
    };
}
