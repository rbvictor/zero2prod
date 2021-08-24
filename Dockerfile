FROM rust:1.54.0 as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.54.0 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

# We use the latest Rust stable release as base image
FROM rust:1.54.0 AS builder

# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app

# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Copy all files from our working environment to our Docker image
COPY . .

# Let's set the SQLX_OFFLINE environment variable to true in our Dockerfile
# to force sqlx to look at the saved metadata instead of trying to query a live database
ENV SQLX_OFFLINE true

# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release

# Runtime stage
FROM debian:buster-slim AS runtime

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod

# We need the configuration file at runtime!
COPY configuration configuration

ENV APP_ENVIRONMENT production

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]
