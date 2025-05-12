Configs for use in production.

Each directory must have a `manifest.toml` file with a `FILES = [ .. ]` array that lists other config files in the same directory. This is a workaround to WASM being unable to iterate directory contents.

Each high-level binary crate has a separate `config` directory where config overrides can be specified for testing purposes.
