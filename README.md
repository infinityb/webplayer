## webplayer

[![Build Status](https://travis-ci.org/infinityb/webplayer.svg?branch=master)](https://travis-ci.org/infinityb/webplayer)

What if you could listen to your music at work?

## Running

Works on nightly (Rocket.rs dependency)

    cargo run --release -- ./Config.toml

## Config.toml

    secret = "put random secrets here"

    [google_auth]
    # get one of these here https://console.developers.google.com
    audience = "random-hexadecimal.apps.googleusercontent.com"

    [database]
    write_url = "postgresql://musicapp:ayy@localhost:5432/musicapp"

## Docker

The provided `docker-compose.yml` starts a PostgreSQL instance bound to port 5432 and the webplayer at port 8000.

The created PostgreSQL user:password is `musicapp:ayy`. A database called `musicapp` is also created.

It reads a configuration from `./Config.toml`

    docker-compose up

## License

webplayer is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
