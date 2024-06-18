# Small Backup (Smackup)
Smackup is a small backup is a minimal backup solution that supports flat files and can replicate them to different locations. 

Via the an `.env file you can set replicated folders (I use it to replicate to multiple HDDs/SSDs, you can do just 1). 

All files between all folders are synced. Also it has an easy to use web frontend.
settings currently are:

```
REPLICATION_LOCATIONS=./data/data1:./data/data2 # Replication locations separated by ":"
PORT=4000 # Port where the app runs
GENERATE_DIRS=False # creates the folders that are set as replication when set to "True"
```

![image](https://github.com/corgijan/little-share/assets/70795482/918a0b23-b00d-44b9-8c97-1a659e6e1596)

## Cross compile for RASPI
```shell
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:armv7-musleabihf'
rust-musl-builder cargo build --release
```

```shell
scp target/armv7-unknown-linux-musleabihf/release/little-share user@pi:/home/user/location
```

