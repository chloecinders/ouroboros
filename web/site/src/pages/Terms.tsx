export function Terms(props: { updated: string }) {
    return (
        <div class="document-page">
            <h1>Terms of Service</h1>
            <div class="last-updated">Last Updated: {props.updated}</div>

            <h2>1. Acceptance of Terms</h2>
            <p>
                By inviting the Aegis Discord bot ("the bot") to your Discord server or using its features, you agree to
                be bound by these Terms of Service. If you do not agree to these terms, you must immediately remove the
                Bot from your server and cease its use. Server administrators are required to disclose the usage of the
                bot for the purposes of user data transparency, especially in case of logging being active.
            </p>

            <h2>2. Description of Service</h2>
            <p>
                Aegis is a specialized utility bot focused on moderation tasks within Discord communities. It helps
                server administrators and moderators in moderation of their Discord server by providing commands which
                take action against users, automatic moderation through the use of message content scanning and image
                text scanning and logging of server activity, especially user messages.
            </p>

            <h2>3. Server Owner Responsibilities</h2>
            <p>
                As the server owner or a server administrator utilizing Aegis, you are solely responsible for
                configuring the Bot correctly, such as ensuring the right people have permissions to access specific
                commands and that user data is not exposed publicly.
            </p>

            <h2>4. Prohibited Uses</h2>
            <p>
                You agree not to use Aegis in a manner that violates Discord's Terms of Service, harasses users
                maliciously or leads to disruption of the bot's services which can affect other users of the bot.
            </p>

            <h2>5. Open Source and Self-Hosting</h2>
            <p>
                The underlying source code of Aegis is provided under its respective open-source license. You are free
                to self-host and use the bot in compliance with that license. You are solely required to provide your
                own terms of service and privacy policy, as this document and the Aegis privacy policy found under
                http://aegis.chloecinders.com/privacy does not cover self-hosted instances.
            </p>

            <h2>6. Disclaimer of Warranties</h2>
            <p>
                THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
                LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN
                NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
                WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
                SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
            </p>

            <h2>7. Changes to the Terms</h2>
            <p>
                We reserve the right to update or modify these Terms of Service at any time. The date of the last update
                is shown at the top of this page.
            </p>
        </div>
    );
}
