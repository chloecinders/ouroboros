import type { Accessor } from "solid-js";
import { createStore, produce } from "solid-js/store";

import type { Offered } from "../../api.ts";
import { call } from "../../api.ts";
import { ask, discard } from "../ask.ts";
import type { Editing } from "./editing.ts";
import type { Focus } from "./focus.ts";
import type { Rules } from "./rules.ts";
import type { Session } from "./session.ts";
import { trouble } from "./session.ts";

interface Answered {
    mode: string | null;
    response: string;
}

export interface Taken extends Offered {
    saved: Answered;
}

export interface Managed {
    list: Taken[];
    open: Accessor<Taken | undefined>;
    unsaved: Accessor<boolean>;
    enable: (mode: string) => void;
    take: (offers: Offered[]) => void;
    show: (id: string) => Promise<void>;
    leaving: () => Promise<boolean>;
    subscribe: () => Promise<void>;
    keep: () => void;
    restore: () => void;
    leave: () => Promise<void>;
}

const answered = (from: Answered): Answered => ({ mode: from.mode, response: from.response || "" });

export const adoptOffer = (offer: Offered): Taken => Object.assign({}, offer, { saved: answered(offer) });

export function createManaged(session: Session, focus: Focus, editing: Editing, rules: Rules): Managed {
    const [list, setList] = createStore<Taken[]>([]);

    const open = () => list.find((offer) => offer.id === focus.managed());
    const index = () => list.findIndex((offer) => offer.id === focus.managed());

    function unsaved() {
        const offer = open();

        if (!offer || !offer.mode) return false;

        return offer.mode !== offer.saved.mode || editing.draft() !== offer.saved.response;
    }

    async function leaving() {
        const offer = open();

        return !unsaved() || !offer || (await discard(offer.name));
    }

    async function push(offer: Taken, written: Answered, telling: string) {
        session.say(telling, "");

        const answer = await session.during(() => call<Offered>(session.where.offer(offer.id), "PUT", written));

        if (answer.error) {
            session.say(trouble(answer), "bad");

            return null;
        }

        setList(index(), adoptOffer(answer.value));
        session.say("saved", "");

        return answer.value;
    }

    return {
        list,
        open,
        unsaved,
        enable: (mode) => setList(index(), "mode", mode),
        take: (offers) => setList(offers.map(adoptOffer)),
        async show(id) {
            const rule = rules.open();

            if (id === focus.managed()) return;

            if (rules.dirty() && rule && !(await discard(rule.name))) return;

            if (!(await leaving())) return;

            focus.onManaged(id);
            editing.load(list.find((offer) => offer.id === id)?.response ?? "", "response");
            session.surface.name?.blur();
            session.say(null);
            editing.check();
        },
        leaving,
        async subscribe() {
            const offer = open();

            if (!offer || session.busy()) return;

            const taken = await push(offer, { mode: "disabled", response: "then delete" }, "subscribing");

            if (taken) editing.load(taken.response, "response");
        },
        keep() {
            const offer = open();

            if (!offer || session.busy() || !unsaved()) return;

            push(offer, { mode: offer.mode, response: editing.draft() }, "saving");
        },
        restore() {
            const offer = open();

            if (!offer) return;

            const back = offer.saved;

            setList(
                index(),
                produce((slot) => Object.assign(slot, answered(slot.saved))),
            );

            editing.load(back.response, "response");
            session.say(null);
            editing.check();
        },
        async leave() {
            const offer = open();

            if (!offer || session.busy()) return;

            const yes = await ask({
                headline: "unsubscribe from " + offer.name + "?",
                confirm: "unsubscribe",
                danger: true,
            });

            if (!yes) return;

            const answer = await session.during(() => call<null>(session.where.offer(offer.id), "DELETE"));

            if (answer.error) return session.say(trouble(answer), "bad");

            setList(
                index(),
                adoptOffer(Object.assign({}, offer, { mode: null, effective: "disabled", response: "", action: null })),
            );

            editing.load("", "response");
            session.say("unsubscribed from " + offer.name, "");
        },
    };
}
