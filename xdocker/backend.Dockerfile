## Stage: build executables
FROM rust:alpine AS build
WORKDIR /builds

# cache dependencies
RUN mkdir bins \
    && mkdir bins/game_instance \
    && mkdir bins/game_instance/src \
    && echo 'fn main() { panic!("Dummy Image Called!")}' > bins/game_instance/src/main.rs
COPY bins/game_instance/Cargo.toml bins/game_instance

RUN mkdir bins/backend \
    && mkdir bins/backend/src \
    && echo 'fn main() { panic!("Dummy Image Called!")}' > bins/backend/src/main.rs
COPY bins/backend/Cargo.toml bins/backend

COPY Cargo.toml Cargo.lock ./
COPY ./libs ./libs

RUN apk add --no-cache musl-dev
RUN cargo build --no-default-features --profile release-unoptimized -p game_instance
RUN cargo build --no-default-features --profile release-unoptimized -p backend

# build executables
COPY ./bins/game_instance/src ./bins/game_instance/src
COPY ./bins/backend/src ./bins/backend/src
RUN touch ./bins/game_instance/src/main.rs  # break cargo cache
RUN touch ./bins/backend/src/main.rs  # break cargo cache
RUN cargo build --no-default-features --profile release-unoptimized -p game_instance
RUN cargo build --no-default-features --profile release-unoptimized -p backend


## Stage: save executables
FROM alpine:latest AS runner
COPY --from=build /builds/target/release-unoptimized/game_instance /usr/bin/game_instance
COPY --from=build /builds/target/release-unoptimized/backend /usr/bin/backend
RUN mkdir usr/assets
ENV BEVY_ASSET_ROOT=usr/assets

# host-user server
# TODO: inject the proxy ip and domain name from host
# NOTE: must bind-mount the certs from the host to the same directory in the container
# example: docker run --rm --network host --mount type=bind,readonly,src=/etc/letsencrypt/,dst=/etc/letsencrypt
# snowyroadstudios/prov_backend
CMD [\
    "backend",\
    "--game-instance", "usr/bin/game_instance",\
    "--host-addr", "0.0.0.0:48888",\
    "--local-ip", "0.0.0.0",\
    "--proxy-ip", "159.89.48.217",\
    "--ws-domain", "providence-prealpha-backend-0.online",\
    "--wss-certs", "etc/letsencrypt/live/providence-prealpha-backend-0.online/fullchain.pem",\
    "--wss-certs-privkey", "etc/letsencrypt/live/providence-prealpha-backend-0.online/privkey.pem"\
]
