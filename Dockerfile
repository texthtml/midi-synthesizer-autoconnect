FROM buildpack-deps:bullseye AS builder

RUN apt-get update && apt-get install -y curl build-essential

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN . $HOME/.cargo/env && \
    rustup component add cargo

RUN apt-get update && apt-get install -y libasound2-dev

WORKDIR /app

ADD .cargo /app/.cargo/
ADD Cargo.toml Cargo.lock .cargo /app/
ADD vendor /app/vendor/

RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    . $HOME/.cargo/env && \
    cargo build --release --offline

ADD src /app/src/

RUN touch src/main.rs && \
    . $HOME/.cargo/env && \
    cargo build --release --offline

FROM debian:bullseye

RUN apt-get update && \
    apt-get install -y libasound2 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/midi-synthesizer-autoconnect /usr/bin/

CMD ["/usr/bin/midi-synthesizer-autoconnect"]
