import { For, Show, createResource, createSignal } from "solid-js";

import type { Permission, Refusal, View, Vocabulary } from "../api.ts";
import { API, ask, post, put, remove, wording } from "../api.ts";
import { useGuild } from "./Guild.tsx";
import { Explain, Note } from "./Note.tsx";

interface Loaded {
    error?: Refusal;
    rules: Permission[];
    vocabulary: Vocabulary;
}

interface Said {
    text: string;
    kind: string;
}

const find = (listed: { id: string; name: string }[], id: string) => listed.find((one) => one.id === id);

function label(guild: View, scope: string, id: string): string {
    if (scope === "role") {
        const role = find(guild.roles, id);

        return role ? "@" + role.name : "unknown role " + id;
    }

    if (scope === "channel") {
        const channel = find(guild.channels, id);

        return channel ? "#" + channel.name : "unknown channel " + id;
    }

    return "member " + id;
}

function Row(props: {
    rule: Permission;
    guild: View;
    confirming: boolean;
    onConfirm: (id: number) => void;
    onRemove: (id: number) => void;
    onRank: (id: number, priority: number) => void;
}) {
    function rank(field: HTMLInputElement) {
        const written = Number(field.value.trim());

        if (!Number.isInteger(written)) {
            field.value = String(props.rule.priority);
            return;
        }

        if (written !== props.rule.priority) props.onRank(props.rule.id, written);
    }

    return (
        <div class="permits__row">
            <span class="permits__id">#{props.rule.id}</span>
            <span class={"pill pill--" + props.rule.effect}>{props.rule.effect}</span>
            <span class="permits__target">{props.rule.target}</span>
            <span class="permits__subject">{label(props.guild, props.rule.scope, props.rule.subject)}</span>

            <input
                class="permits__priority"
                value={props.rule.priority}
                inputmode="numeric"
                onChange={(e) => rank(e.currentTarget)}
            />

            <Show
                when={props.confirming}
                fallback={
                    <button class="permits__remove" onClick={() => props.onConfirm(props.rule.id)}>
                        remove
                    </button>
                }>
                <button class="permits__remove permits__remove--sure" onClick={() => props.onRemove(props.rule.id)}>
                    confirm
                </button>
            </Show>
        </div>
    );
}

function Grant(props: { guild: View; vocabulary: Vocabulary; onWritten: () => void }) {
    const [effect, setEffect] = createSignal("allow");
    const [scope, setScope] = createSignal("role");
    const [subject, setSubject] = createSignal("");
    const [target, setTarget] = createSignal("*");
    const [priority, setPriority] = createSignal("0");
    const [busy, setBusy] = createSignal(false);
    const [said, setSaid] = createSignal<Said | null>(null);

    const subjects = () => (scope() === "role" ? props.guild.roles : props.guild.channels);

    async function save() {
        if (!subject()) {
            setSaid({ text: "provide a subject", kind: "bad" });
            return;
        }

        const ranking = Number(priority().trim() || "0");

        if (!Number.isInteger(ranking)) {
            setSaid({ text: "expected a whole number", kind: "bad" });
            return;
        }

        setBusy(true);
        setSaid({ text: "saving", kind: "" });

        const answer = await post<Permission>(API.permissions(props.guild.id), {
            scope: scope(),
            subject: subject(),
            target: target(),
            effect: effect(),
            priority: ranking,
        });

        setBusy(false);

        if (answer.error) {
            setSaid({ text: answer.detail ? answer.detail.problem : wording(answer.error), kind: "bad" });
            return;
        }

        setTarget("*");
        setPriority("0");
        setSaid({ text: "saved", kind: "ok" });
        props.onWritten();
    }

    return (
        <div class="grant">
            <div class="grant__line">
                <label class="grant__label">
                    <span class="grant__title">effect</span>
                    <select class="grant__select" value={effect()} onChange={(e) => setEffect(e.currentTarget.value)}>
                        <option value="allow">allow</option>
                        <option value="deny">deny</option>
                    </select>
                </label>

                <label class="grant__label">
                    <span class="grant__title">scope</span>
                    <select
                        class="grant__select"
                        value={scope()}
                        onChange={(e) => {
                            setScope(e.currentTarget.value);
                            setSubject("");
                        }}>
                        <option value="role">role</option>
                        <option value="member">member</option>
                        <option value="channel">channel</option>
                    </select>
                </label>

                <label class="grant__label grant__label--wide">
                    <span class="grant__title">subject</span>

                    <Show
                        when={scope() !== "member"}
                        fallback={
                            <input
                                class="grant__input"
                                value={subject()}
                                placeholder="member id"
                                inputmode="numeric"
                                onInput={(e) => setSubject(e.currentTarget.value.trim())}
                            />
                        }>
                        <select class="grant__select" value={subject()} onChange={(e) => setSubject(e.currentTarget.value)}>
                            <option value="">unset</option>

                            <For each={subjects()}>
                                {(one) => <option value={one.id}>{(scope() === "role" ? "@" : "#") + one.name}</option>}
                            </For>
                        </select>
                    </Show>
                </label>

                <label class="grant__label grant__label--wide">
                    <span class="grant__title">target</span>
                    <select class="grant__select" value={target()} onChange={(e) => setTarget(e.currentTarget.value)}>
                        <option value="*">everything</option>

                        <optgroup label="whole categories">
                            <For each={props.vocabulary.categories}>
                                {(category) => <option value={"@" + category}>@{category}</option>}
                            </For>
                        </optgroup>

                        <For each={props.vocabulary.categories}>
                            {(category) => (
                                <optgroup label={category}>
                                    <For each={props.vocabulary.commands.filter((command) => command.category === category)}>
                                        {(command) => <option value={command.name}>{command.name}</option>}
                                    </For>
                                </optgroup>
                            )}
                        </For>
                    </select>
                </label>

                <label class="grant__label grant__label--narrow">
                    <span class="grant__title">priority</span>
                    <input
                        class="grant__input"
                        value={priority()}
                        inputmode="numeric"
                        onInput={(e) => setPriority(e.currentTarget.value)}
                    />
                </label>
            </div>

            <div class="grant__line grant__line--end">
                <span class={said() ? "grant__said grant__said--" + said()?.kind : "grant__said"}>{said() ? said()?.text : ""}</span>

                <button class="dashboard__button" disabled={busy()} onClick={save}>
                    save
                </button>
            </div>
        </div>
    );
}

export function Permits() {
    const guild = useGuild();
    const [loaded, { refetch }] = createResource(
        () => guild().id,
        async (id: string): Promise<Loaded> => {
            const [rules, vocabulary] = await Promise.all([
                ask<Permission[]>(API.permissions(id)),
                ask<Vocabulary>(API.commands),
            ]);

            const refused = rules.error || vocabulary.error;

            if (refused) return { error: refused } as Loaded;

            return { rules: rules.value, vocabulary: vocabulary.value };
        },
    );
    const [confirming, setConfirming] = createSignal<number | null>(null);

    async function revoke(id: number) {
        setConfirming(null);

        await remove(API.permission(guild().id, id));

        refetch();
    }

    async function rank(id: number, priority: number) {
        await put(API.permission(guild().id, id), { priority });

        refetch();
    }

    return (
        <Show when={loaded()}>
            {(loaded) => (
                <Show when={!loaded().error} fallback={<Explain error={loaded().error} />}>
                    <div class="section">
                        <span>permissions</span>
                    </div>

                    <Show
                        when={loaded().rules.length}
                        fallback={
                            <Note headline="no permission rules found" />
                        }>
                        <div class="permits">
                            <div class="permits__head">
                                <span>rule</span>
                                <span>effect</span>
                                <span>target</span>
                                <span>subject</span>
                                <span>priority</span>
                                <span />
                            </div>

                            <For each={loaded().rules}>
                                {(rule) => (
                                    <Row
                                        rule={rule}
                                        guild={guild()}
                                        confirming={confirming() === rule.id}
                                        onConfirm={(id) => setConfirming(id)}
                                        onRemove={revoke}
                                        onRank={rank}
                                    />
                                )}
                            </For>
                        </div>

                        <p class="permits__aside">
                            listed in the order they resolve. highest priority first, then most specific: member over
                            role, role over channel.
                        </p>
                    </Show>

                    <div class="section">
                        <span>new rule</span>
                    </div>

                    <Grant guild={guild()} vocabulary={loaded().vocabulary} onWritten={refetch} />
                </Show>
            )}
        </Show>
    );
}
