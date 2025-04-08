High-level binary for local playtesting.

The `config` directory can be used to override production configs. Requires the `dev` feature.

Run 2-player with:
```
cargo build -p game_client -f dev &&
cargo build -p game_instance -f dev &&
cargo run -p playtest -f dev -- --clients 2
```
