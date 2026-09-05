import type { Opened } from "./api.ts";
import { API, STAMP, hold } from "./api.ts";

const RUNTIME = "/activity.js";

function within<T>(work: Promise<T>): Promise<T> {
    return Promise.race([
        work,
        new Promise<T>((_, refuse) => setTimeout(() => refuse(new Error("no Discord response")), 15000)),
    ]);
}

export async function handshake(): Promise<string | null> {
    const sdk = (await import(RUNTIME)) as typeof import("../activity.ts");

    const { code, guild } = await within(sdk.enter(STAMP.client));

    const answer = await fetch(API.activity, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ code }),
    });

    if (!answer.ok) throw new Error("session request failed");

    hold(((await answer.json()) as Opened).token);

    return guild;
}
