import { useEditor } from "../state/editor.tsx";

export function Description() {
    const { session, rules } = useEditor();

    const written = () => rules.open()?.description ?? "";
    const left = () => 300 - written().length;

    return (
        <>
            <div class="colhead colhead--grey">
                <span>description</span>
                <span class={left() < 0 ? "colhead__count colhead__count--over" : "colhead__count"}>{left()}</span>
            </div>

            <div class="pad">
                <textarea
                    class="description"
                    rows={2}
                    spellcheck={true}
                    value={written()}
                    disabled={!rules.open()}
                    onInput={(e) => {
                        rules.describe(e.currentTarget.value);
                        session.say(null);
                    }}
                />
            </div>
        </>
    );
}
