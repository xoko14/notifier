FROM rust:alpine AS builder

WORKDIR /app

COPY . .

RUN apk add pkgconfig openssl-dev libc-dev

RUN cargo build --manifest-path ./crates/notifier-server/Cargo.toml --release

FROM alpine:latest

WORKDIR /app

COPY --from=builder /app/target/release/notifier-server .

EXPOSE 3000

CMD ["/app/notifier-server"]
