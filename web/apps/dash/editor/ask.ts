import { createSignal } from "solid-js";

export interface Asking {
    headline: string;
    body?: string;
    confirm: string;
    danger?: boolean;
}

interface Open extends Asking {
    reply: (yes: boolean) => void;
}

const [pending, setPending] = createSignal<Open | null>(null);

export const question = pending;

export function ask(asking: Asking): Promise<boolean> {
    return new Promise((reply) => {
        const open = pending();

        if (open) open.reply(false);

        setPending({ ...asking, reply });
    });
}

export function answer(yes: boolean) {
    const open = pending();

    if (!open) return;

    setPending(null);
    open.reply(yes);
}

export const discard = (name: string) =>
    ask({
        headline: "discard changes?",
        body: "unsaved edits on " + name,
        confirm: "discard",
        danger: true,
    });
