# Use the official Rust image
FROM rust:latest

# Set the working directory
WORKDIR /usr/src/l1_poc

# Copy the project files
COPY . .

# Build the project
RUN cargo build --release

# Run the binary
CMD ["./target/release/l1_poc"]
