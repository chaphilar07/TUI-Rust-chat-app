#Note that we are using a planning phase as well here, 
#the cargo chef is basically doing the planning phase, this means that we compile the dependencies, then we will 
#cache those layers so that we can reuse those cached layers and therefore we know that we can speed up the creation of
#future images

#Note that it is very expensive to install the dpendencies for the application in rust, therefore we want to be able to reuse those layers
#if ever we want to reuse those dependencies for a future image, basically creates cached reusable layers.
FROM rust:1.87-slim AS planner
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libsqlite3-dev pkg-config
RUN cargo install cargo-chef

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.87-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libsqlite3-dev pkg-config
RUN cargo install cargo-chef

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json   # builds deps only

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/server .
COPY --from=builder /app/Users.db ./

EXPOSE 42069
CMD ["./server"]
