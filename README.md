# Small Backup (Smackup)
Smackup is a small Backup solution that supports just files. Via the ENV you can set replicated folders (I use it to replicate to multiple HDDs/SSDs). All files between all folders are synced. Also it has an eassy web frontend.


```shell
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:armv7-musleabihf'                                                  I
rust-musl-builder cargo build --release
```

```shell
scp target/armv7-unknown-linux-musleabihf/release/little-share user@pi:/home/user/location
```
![image](https://github.com/corgijan/little-share/assets/70795482/918a0b23-b00d-44b9-8c97-1a659e6e1596)

