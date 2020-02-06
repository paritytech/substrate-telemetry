FROM rust

WORKDIR /app

COPY ./backend .

RUN cargo build --release

EXPOSE 8000

ENTRYPOINT [ "./target/release/telemetry" ]