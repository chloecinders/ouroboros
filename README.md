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
database_url = "postgres:#user:password@ip/database" # database url
max_connections = 5 # database max connections

[dev]
token = ""
prefix = "!"
database_url = ""
max_connections = 5
```
