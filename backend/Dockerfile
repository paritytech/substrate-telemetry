#### BUILDER IMAGE -- quite big one ####
FROM fredrikfornwall/rust-static-builder as builder
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry backend build image"

ARG PROFILE=release

WORKDIR /app

COPY . .

RUN cargo build --${PROFILE} --bins --target x86_64-unknown-linux-musl


# MAIN IMAGE FOR PEOPLE TO PULL --- small one#
FROM alpine
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry backend"

ARG PROFILE=release

COPY --from=builder /app/target/x86_64-unknown-linux-musl/$PROFILE/telemetry /usr/local/bin

EXPOSE 8000

ENTRYPOINT ["/usr/local/bin/telemetry"]