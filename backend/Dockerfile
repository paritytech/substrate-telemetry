#### BUILDER IMAGE -- quite big one ####
FROM rust as builder
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry backend build image"

ARG PROFILE=release

WORKDIR /app

COPY . .

RUN cargo build --$PROFILE


# MAIN IMAGE FOR PEOPLE TO PULL --- small one#
FROM alpine
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry backend"

ARG PROFILE=release

COPY --from=builder /app/target/$PROFILE/telemetry /usr/local/bin

EXPOSE 8000

ENTRYPOINT ["/usr/local/bin/telemetry"]
