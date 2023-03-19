FROM rust as build
WORKDIR /build
ARG GIT_SHA


RUN apt update
RUN apt install libssl-dev -y

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src ./src

RUN cargo build --release
RUN chmod a+x /build/target/release/strecken-info-telegram

FROM rust
WORKDIR /app

ENV PATH="$PATH:/app/bin"

COPY --from=build /build/target/release/strecken-info-telegram /app/bin/strecken-info-telegram

ENV SQLITE_PATH=/database/db.sql
VOLUME [ "/database" ]

CMD ["./bin/strecken-info-telegram"]