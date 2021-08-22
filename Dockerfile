FROM rust:1.54-bullseye AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y libasound2-dev

ADD . /app/

RUN cargo build --release

FROM debian:bullseye

RUN apt-get update && \
    apt-get install -y libasound2 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/midi-synthetizer-autoconnect /usr/bin/

CMD ["/usr/bin/midi-synthetizer-autoconnect"]
