# Ouroboros

Discord mod bot or something

dependencies:
cargo/rustc 1.89.0-nightly

install:
- grab the latest binary for your os from the latest build artifact
- create a Config.toml in the same directory as the binary (see below)
- run the binary
- win

Config format:
```toml
[bot]
env = "release" # either release or dev to set which environment it pulls from

[release]
token = "" # bot token
prefix = "!" # bot prefix
database_url = "postgres://user:password@ip/database" # database url, must be postgres
max_connections = 5 # database max connections
repository = "chloecinders/ouroboros" # the repository to update from
github_token = "" # the github token with actions access to the repository in case its private
dev_ids = [1234567890] # list of user ids which have access to developer commands
whitelist_enabled = true # enables the whitelist
whitelist = [9876541321, 1234567890] # list of whitelisted server ids

# required config is here (note that dev can still accept all the settings above)
[dev]
token = ""
prefix = "!"
database_url = ""
max_connections = 5
```
