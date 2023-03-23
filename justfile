default:
  @just --choose

# Push code to Space and trigger build
push:
  space push

# Run the application
run:
  cargo run

# Run local dev
dev:
  spacex dev

# Eg. `update_space v0.0.10-rc5`
update_space tag bin="spacex":
  eget deta/space-cli --tag={{ tag }} \
  --file=space \
  --to=$HOME/.detaspace/bin/{{ bin }}


# Run shellcheck on scripts folder
shellcheck severity="warning":
  docker run --rm -v "$PWD:/mnt" koalaman/shellcheck:stable -x -P SCRIPTDIR -S {{ severity }} scripts/*
