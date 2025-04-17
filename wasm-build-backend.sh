# Building for WASM
# TODO: find better way to input host server address
cp -r config/frontend wasm/client/config
cp -r config/user_client wasm/client/config
cp -r config/game wasm/client/config
cp -r config/game_client wasm/client/config
cp -r assets wasm/client
PROV_HOST_ADDR=providence-prealpha-backend-0.online:48888 PROV_HOST_IS_WSS=true PROV_CONFIG_DIR=config cargo build -p client no-default-features --target wasm32-unknown-unknown --release
wasm-bindgen --no-typescript --out-name prov_client --out-dir wasm/client --target web target/wasm32-unknown-unknown/release/client.wasm
wasm-opt --all-features -Os wasm/client/prov_client_bg.wasm -o wasm/client/prov_client_bg.wasm
zip -r xbuilds/prov_client.zip wasm/client
