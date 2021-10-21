FROM alpine AS builder

RUN apk add cargo --repository=http://dl-cdn.alpinelinux.org/alpine/edge/testing/

RUN apk add alsa-lib-dev

WORKDIR /app

ADD .cargo /app/.cargo/
ADD Cargo.toml Cargo.lock .cargo /app/
ADD vendor /app/vendor/

RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release --offline

ADD src /app/src/

RUN touch src/main.rs && \
    cargo build --release --offline

FROM alpine

RUN apk add --no-cache libgcc alsa-lib

COPY --from=builder /app/target/release/midi-synthesizer-autoconnect /usr/bin/

CMD ["/usr/bin/midi-synthesizer-autoconnect"]
