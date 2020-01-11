# root
This is simple binary for testing some of the external lib features

## Run docker image for dependency checks
```bash
# Build the docker image and copy in the pre-built binanry at `target/release/root`
$ docker build -t fungus:latest .

# Run the docker image and test the binary
$ docker run --rm -it fungus:latest bash
```
