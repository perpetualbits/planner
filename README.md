# Amity — a peaceful home

Amity is a household planner that runs on hardware the household owns. It holds
the things a mind would otherwise hold — chores, events, meals, the small
ongoing threads of family life — so the people in the home don't have to.

It is a kitchen device, a mobile companion, a quiet piece of infrastructure. It
is not a productivity app, a social network, an assistant, or a coach. It is a
place where things rest.

Read [the philosophy](docs/amity_philosophy.md) to understand why this project
exists and what it refuses to become. Read [the brief](docs/amity_brief.md) for
the full design and data model.

## Getting started

```
cargo build --workspace
cargo test --workspace
cargo run --bin amity-service
```

The service reads `$XDG_CONFIG_HOME/amity/config.toml` (Linux) or the
platform equivalent; it falls back to built-in defaults if the file is absent.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the DCO sign-off requirement, commit
format, and how to run the test suite.
