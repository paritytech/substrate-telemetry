FROM rust as builder

WORKDIR /build
COPY ./backend .
RUN cargo build --release

#####################################
FROM phusion/baseimage:0.11

COPY --from=builder /build/target/release/telemetry /usr/local/bin

EXPOSE 8000

ENTRYPOINT [ "telemetry" ]
