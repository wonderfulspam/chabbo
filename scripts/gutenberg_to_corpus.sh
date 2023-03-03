#!/bin/bash

## GUTENBERG TO CORPUS ##
# Takes a link to plain UTF-8 text file and produces a stripped down text file
# for use in Chabbo

DIR="$(dirname "$(realpath "$0")")"
# shellcheck source=helpers.sh
source "${DIR}/helpers.sh"

INPUT_URL=$1
OUTPUT_FILE=$2

# Sanity checks
test_or_exit [[ $# -eq 2 ]] "Must provide two input arguments: URL and output file"
check_executable_exists cat
check_executable_exists wget
check_executable_exists sd

echo "Downloading $INPUT_URL and creating resulting file at $OUTPUT_FILE"
read -p "Are you sure? " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    result=$(wget -qO- "$INPUT_URL")
    echo "$result" | \
    # Remove any text before "*** START OF"
    sd -f ms ".*\*\*\* START OF" '' | \
    # Remove any text after "*** END OF"
    sd -f ms "\*\*\* END OF.*" '' | \
    # Remove the last of the "START OF THE PROJECT GUTENBERG" line
    sd "THE PROJECT.*\*\*\*" '' | \
    # Remove all types of quotes
    sd '[“”"]' '' | \
    # Remove carriage returns
    sd "\r+" "" | \
    # Convert newlines to spaces
    sd "\n+" " " | \
    # Break into sentences
    sd '[\?\.;] ' "\n" | \
    # Ensure no double newlines
    sd "\s*\n+" "\n" > "$OUTPUT_FILE"
fi
