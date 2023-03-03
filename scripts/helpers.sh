#!/bin/bash

# Run command, exit if it fails
function test_or_exit() {
    local COMMAND="${@:1:$#-1}"
    local MSG="${@: -1}"

    eval "$COMMAND" || { echo "$MSG"; exit 1; }
}

# Checks whether executable is available in $PATH, exits if not
function check_executable_exists() {
    local EXECUTABLE=$1

    local MSG="Please ensure $EXECUTABLE is installed and available in PATH"
    local COMMAND="command -v $EXECUTABLE >/dev/null 2>&1"

    test_or_exit "$COMMAND" "$MSG"
}
