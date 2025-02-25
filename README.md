# Providence: The Game

Providence is a multiplayer economy builder.


### Full Usage

Run the backend:
```
cargo build -p game_instance && \
cargo run -p backend
```

Run `client 0`:
```
cargo run -p client -- --id 0
```

Run `client 1`:
```
cargo run -p client -- --id 1
```


### Playtest

Run a local multiplayer game from the command line:
```
cargo build -p game_client && \
cargo build -p game_instance && \
cargo run -p playtest -- --clients 2
```
