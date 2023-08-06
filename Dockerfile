FROM rust:slim AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /usr/src
RUN cargo new auth-adapter

WORKDIR /usr/src/auth-adapter
COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN sed -i 's/"migration", //g' Cargo.toml

RUN cargo new --lib entities

COPY ./entities/Cargo.toml entities

RUN cargo fetch

COPY . .
RUN sed -i 's/"migration", //g' Cargo.toml

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
COPY --from=builder /usr/src/auth-adapter/target/x86_64-unknown-linux-musl/release/auth-adapter ./
CMD [ "./auth-adapter" ]
