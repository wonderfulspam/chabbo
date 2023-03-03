default:
  @just --choose

# Build for Deta target
build:
  cargo build --target x86_64-unknown-linux-gnu --release

# Build, move to bin and strip debug symbols from binary
package: build
  cp target/x86_64-unknown-linux-gnu/release/chabbo bin/
  strip bin/chabbo

# Build, package and push
push: package
  space push

# Run the application
run:
  cargo run
