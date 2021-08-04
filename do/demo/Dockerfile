FROM rust:latest

WORKDIR /app
COPY . /app
RUN cargo build -p demo --release

ENTRYPOINT ["/app/target/release/demo"]
