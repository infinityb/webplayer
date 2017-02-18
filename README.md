## webplayer

[![Build Status](https://travis-ci.org/infinityb/webplayer.svg?branch=master)](https://travis-ci.org/infinityb/webplayer)

What if you could listen to your music at work?

## Docker

The provided `docker-compose.yml` starts a PostgreSQL instance bound to port 5432 and the webplayer at port 8000.

The created PostgreSQL user:password is `musicapp:ayy`. A database called `musicapp` is also created.

    docker-compose up

## License

webplayer is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
