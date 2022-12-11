FROM ubuntu:22.04

# Install the necessary packages to build and run Rust programs
RUN apt-get update && \
    apt-get install -y curl build-essential libssl-dev 

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

# Make sure cargo/bin is in the path
ENV PATH=$PATH:/root/.cargo/bin

# Copy files to the Docker image at /src
COPY . /src

# Build CloudflareDDNS as a release
RUN cd /src && cargo build --release

# Set the working directory to where the binary was generated
WORKDIR /src/target/release

# Make the binary run when the container is started
CMD ["/src/target/release/cloudflareddns"]