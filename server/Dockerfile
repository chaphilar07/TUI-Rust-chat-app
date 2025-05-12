
#The builder phase of the Dockerfile, 
FROM --platform=linux/arm64 rust:1.82 AS builder
WORKDIR /app



#Copy the entire contents of the context
COPY . ./
COPY target/release/server ./


#Install extra dependencies.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      libsqlite3-dev \
      sqlite3 \
      pkg-config \
 && rm -rf /var/lib/apt/lists/*

#Get rid of the old users and create a fresh version.
RUN rm -r Users.db
RUN sqlite3 Users.db < .schema.sql


#Install sqlx get cli features,
RUN cargo install --locked sqlx-cli --no-default-features --features sqlite



RUN DATABASE_URL="sqlite:Users.db" \
    cargo sqlx prepare 
RUN cargo build -p server --release



FROM --platform=linux/arm64 debian:bookworm AS runtime
WORKDIR /app 
COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/Users.db ./Users.db

ENTRYPOINT ["/usr/local/bin/server"]


EXPOSE 42069







