FROM --platform=linux/arm64 rust:1.82 AS builder
WORKDIR /app


COPY . ./
COPY target/release/client ./ 

RUN cargo build -p client --release


FROM --platform=linux/arm64 debian:bookworm AS runtime
WORKDIR /usr/local/bin 


COPY --from=builder /app/target/release/client /usr/local/bin/client



ENTRYPOINT ["/usr/local/bin/client"]
