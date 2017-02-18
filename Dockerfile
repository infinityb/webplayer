FROM liuchong/rustup:nightly

WORKDIR /app/src

RUN rustup override set nightly-2017-02-17

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

COPY . ./
RUN cargo build --release --locked

EXPOSE 8000

ENTRYPOINT ["cargo", "run", "--release", "--", "Config.toml"]
CMD ["--"]
