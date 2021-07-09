FROM paritytech/ci-linux:production as builder

ARG PROFILE=release
WORKDIR /app

COPY . .

RUN cargo build --${PROFILE} --bins

# MAIN IMAGE FOR PEOPLE TO PULL --- small one#
FROM debian:buster-slim
LABEL maintainer="Parity Technologies"
LABEL description="Polkadot Telemetry backend shard/core binaries, static build"

ARG PROFILE=release
WORKDIR /usr/local/bin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/$PROFILE/telemetry_shard /usr/local/bin
COPY --from=builder /app/target/$PROFILE/telemetry_core /usr/local/bin
RUN apt-get -y update && apt-get -y install openssl && apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/

EXPOSE 8000

