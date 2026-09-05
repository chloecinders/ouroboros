import { Show } from "solid-js";

export function Crest(props: { name?: string | null; icon?: string | null }) {
    const initials = () =>
        (props.name || "?")
            .split(/\s+/)
            .filter(Boolean)
            .slice(0, 2)
            .map((word) => word[0])
            .join("")
            .toUpperCase();

    return (
        <Show when={props.icon} fallback={<span class="crest">{initials()}</span>}>
            <span class="crest">
                <img class="crest__image" src={props.icon ?? undefined} alt="" />
            </span>
        </Show>
    );
}
