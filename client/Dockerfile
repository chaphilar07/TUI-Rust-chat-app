
# The planner stage we cache the dependencies here for our build.
FROM rust:1.87-slim AS planner

WORKDIR /app
RUN cargo install cargo-chef

COPY . ./ 
RUN cargo chef prepare --recipe-path recipe.json


# The builder, we build the actual project
FROM rust:1.87-slim AS builder

WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --recipe-path recipe.json --release

COPY . ./ 

RUN cargo build --release


# Runtime stage 
FROM debian:bookworm-slim 
RUN apt-get update && apt-get install -y --no-install-recommends 

WORKDIR /app 

COPY --from=builder /app/target/release/client .
CMD ["./client"]

