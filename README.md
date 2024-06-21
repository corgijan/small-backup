# Small Backup (Smackup)
### Smackup is a very minimal backup solution with a web user interface, that supports flat file backups and can replicate them to different locations. 

#### I wanted to have a backup solution for my old pi that is very effiecent and simple enough to hand it to my family and friedns. I couldn't find any so I built it myself.


## Usage
#### The settings are available via an .env next top the binary or via ENV variables. Here is an exampe: 


```
REPLICATION_LOCATIONS=./data/data1:./data/data2 # Replication locations separated by ":"
PORT=4000 # Port where the app runs
GENERATE_DIRS=False # creates the folders that are set as replication when set to "True"
```
## Try it out
```shell
docker run -p 3000:3000 hainz98/smbackup  
```
/data is usually the folder where the backups are stored, if you want to mount something in it, otherwise mount your locations in the /data folder and pass the REPLICATION_LOCATIONS as ENV variables.

for example: 
```shell
docker run -p 3000:3000 -v ./data:/data -e REPLICATION_LOCATIONS=/data/data1:/data/data2 hainz98/smbackup  
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

