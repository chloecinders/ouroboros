import { DiscordSDK } from "@discord/embedded-app-sdk";

let sdk: DiscordSDK | null = null;

export async function enter(clientId: string): Promise<{ code: string; guild: string | null }> {
    const discord = new DiscordSDK(clientId);

    sdk = discord;
    await discord.ready();

    const { code } = await discord.commands.authorize({
        client_id: clientId,
        response_type: "code",
        state: "",
        prompt: "none",
        scope: ["identify", "guilds"],
    });

    return { code, guild: discord.guildId };
}

export function outward(url: string) {
    if (!sdk) throw new Error("sdk not ready");

    return sdk.commands.openExternalLink({ url });
}
