FROM rust:1.54-bullseye AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y libasound2-dev

ADD .cargo /app/.cargo/
ADD Cargo.toml Cargo.lock .cargo /app/
ADD vendor /app/vendor/

RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release --offline

ADD src /app/src/

RUN touch src/main.rs && \
    cargo build --release --offline

FROM debian:bullseye

RUN apt-get update && \
    apt-get install -y libasound2 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/midi-synthesizer-autoconnect /usr/bin/

CMD ["/usr/bin/midi-synthesizer-autoconnect"]
