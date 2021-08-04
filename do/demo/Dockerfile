FROM rust:latest as builder

WORKDIR /app
COPY . /app
RUN cargo build -p demo --release


FROM rust:latest as runtime

WORKDIR /app
COPY --from=builder /app/target/release/demo demo
EXPOSE 6361

ENV RUST_LOG=info

ENTRYPOINT ["/app/demo"]
