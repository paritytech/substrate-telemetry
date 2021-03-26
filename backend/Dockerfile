#### BUILDER IMAGE -- quite big one ####
FROM paritytech/musl-ci-linux as builder
LABEL maintainer="Parity Technologies"
LABEL description="Polkadot Telemetry backend builder image"

ARG PROFILE=release
WORKDIR /app

COPY . .

RUN cargo build --${PROFILE} --bins


# MAIN IMAGE FOR PEOPLE TO PULL --- small one#
FROM scratch
LABEL maintainer="Parity Technologies"
LABEL description="Polkadot Telemetry backend, static build"

ARG PROFILE=release
WORKDIR /usr/local/bin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/$PROFILE/telemetry /usr/local/bin

EXPOSE 8000

ENTRYPOINT ["telemetry"]
