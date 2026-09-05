import { signIn } from "../api.ts";

export function SignIn() {
    return (
        <>
            <div class="band">
                <div />
                <div>
                    <h1 class="band__title">dashboard</h1>
                </div>
            </div>

            <div class="signin">
                <a class="dashboard__button" href={signIn()}>
                    sign in with discord
                </a>
            </div>
        </>
    );
}
