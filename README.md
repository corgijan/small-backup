# Small Backup (Smackup)
### Smackup is a small backup is a very minimal backup solution with a web user interface, that supports flat file backups and can replicate them to different locations. 

#### I wanted to have a backup solution for my old pi that is very effiecent and simple enough to hand it to my family and friedns. I couldn't find any so I built it myself.


### Usage
Via the an `.env file you can set replicated folders (I use it to replicate to multiple HDDs/SSDs, you can do just 1). 

All files between all folders are synced. Also it has an easy to use web frontend.
settings currently are:

```
REPLICATION_LOCATIONS=./data/data1:./data/data2 # Replication locations separated by ":"
PORT=4000 # Port where the app runs
GENERATE_DIRS=False # creates the folders that are set as replication when set to "True"
```
Roadmap 
- [x] directory support
- [ ] s3 (compatible) as location

![image](https://github.com/corgijan/small-backup/assets/70795482/6c39d35c-8055-4501-b7fe-7cdc65fe3015)


## Cross compile for RASPI
```shell
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:armv7-musleabihf'
rust-musl-builder cargo build --release
```

```shell
scp target/armv7-unknown-linux-musleabihf/release/little-share user@pi:/home/user/location
```

