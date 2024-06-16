# Little Share

I needed a little webserver just to share files in my VPN. Maybe ssh would be enough but I like web-frontends. 

```shell
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:armv7-musleabihf'                                                  I
rust-musl-builder cargo build --release
```
```shell
scp target/armv7-unknown-linux-musleabihf/release/little-share user@pi:/home/user/location
```
