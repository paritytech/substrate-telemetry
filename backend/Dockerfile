#### BUILDER IMAGE -- quite big one ####
FROM martinparity/ci-linux-musl:latest as builder
LABEL maintainer="Parity Technologies"
LABEL description="Polkadot Telemetry backend builder image"

ARG PROFILE=release
WORKDIR /app

COPY . .

RUN cargo build --${PROFILE} --bins --target x86_64-unknown-linux-musl

# MAIN IMAGE FOR PEOPLE TO PULL --- small one#
FROM scratch
LABEL maintainer="Parity Technologies"
LABEL description="Polkadot Telemetry backend, static build"

ARG PROFILE=release
WORKDIR /usr/local/bin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/$PROFILE/telemetry /usr/local/bin

EXPOSE 8000

ENTRYPOINT ["telemetry"]
