import type { Offered, Saved, View } from "../../api.ts";
import { call } from "../../api.ts";
import type { Editor, Want } from "./editor.tsx";

async function authored(it: Editor, want: Want) {
    const list = await call<Saved[]>(it.session.where.all);

    if (list.error) return it.session.setRefused(list.error);

    it.session.setView({ id: "0", name: "managed rules", icon: null, roles: [], channels: [] });
    it.rules.take(list.value);

    document.title = "Aegis - managed rules";

    it.session.setReady(true);
    it.rules.show(it.rules.first(want.rule));
}

async function guild(it: Editor, want: Want) {
    const { where } = it.session;

    if (!it.session.guild) return it.session.setRefused("absent");

    const [view, list, offers] = await Promise.all([
        call<View>(where.view),
        call<Saved[]>(where.all),
        call<Offered[]>(where.offers),
    ]);
    const bad = view.error || list.error;

    if (bad) return it.session.setRefused(bad);

    const taken = offers.value || [];
    const managed = want.managed && taken.some((one) => one.id === want.managed) ? want.managed : null;

    it.session.setView(view.value);
    it.managed.take(taken);
    it.rules.take(list.value);

    document.title = "Aegis - " + view.value.name + " automod";

    it.session.setReady(true);

    if (managed) return it.managed.show(managed);

    it.rules.show(it.rules.first(want.rule));
}

export const load = (it: Editor, want: Want) => (it.session.authored ? authored(it, want) : guild(it, want));
