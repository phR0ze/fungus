# root
This is simple binary for live testing both local and in alpine container

## Run docker image for dependency checks
```bash
# Build static binary
$ cargo build --target x86_64-unknown-linux-musl --release

# Build the docker image and copy in the pre-built binanry at `target/release/root`
# Had to add the --no-cache=true to get docker not to cache my binary
$ docker build --no-cache=true -t fungus:latest .

# Run the docker image and test the binary
$ docker run --rm -it --env RUST_BACKTRACE=1 fungus:latest bash
```
