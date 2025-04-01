FROM rust:1

WORKDIR /usr/src/mapper
COPY . .

RUN cargo install --path .

CMD ["mapper"]
