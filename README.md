# Ouroboros

Discord mod bot that is supposed to do one thing and one thing well. Currently in private beta but open source if you want to self host it yourself.

Open source as I believe that people, especially moderators, should know exactly what their tools do.

Feel free to contribute!

Developed and tested on Rust 1.89.0-nightly.
Requires a PostgreSQL database.

install:
- grab the latest binary for your os from the latest build artifact
- create a Config.toml in the same directory as the binary (see below)
- run the binary
- win

update:
dev_id users have access to the commands say and update, update fetches the newest binary from the artifact actions of the specified repository and shut down. If you have systemd to set up on autostart everything is automatic. If you need more specific behaviour feel free to fork the bot!

Config format:
```toml
[bot]
env = "release" # either release or dev to set which environment it pulls settings from

[release]
token = "" # bot token
prefix = "!" # bot prefix
database_url = "postgres://user:password@ip/database" # database url, must be postgres
max_connections = 5 # database max connections
msg_cache = 100 # discord message cache size (per channel)
repository = "chloecinders/ouroboros" # the repository to update from
github_token = "" # the github token with actions access to the repository in case its private
dev_ids = [1234567890] # list of user ids which have access to developer commands
whitelist_enabled = true # enables the whitelist
whitelist = [987654321, 1234567890] # list of whitelisted server ids

# required config is here (note that dev can still accept all the settings above)
[dev]
token = ""
prefix = "!"
database_url = ""
max_connections = 5
msg_cache = 100
```
