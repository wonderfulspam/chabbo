#!/bin/bash

## MESSENGER TO CORPUS ##
# Filters a JSON chat export by author and produces a corpus for use in Chabbo

DIR="$(dirname "$(realpath "$0")")"
source "${DIR}/helpers.sh"

INPUT_FOLDER=$1
OUTPUT_FOLDER=$2
AUTHOR=$3
OUTPUT_FILE="${OUTPUT_FOLDER}/${AUTHOR// /_}.txt" # Convert spaces to underscores
OUTPUT_FILE=${OUTPUT_FILE//\/\//\/} # Remove double slashes

# Sanity checks
test_or_exit [[ $# -eq 3 ]] "Must provide three input arguments: input file, output folder and author"
test_or_exit [[ -d "$INPUT_FOLDER" ]] "$INPUT_FOLDER is not a folder"
test_or_exit [[ -d "$OUTPUT_FOLDER" ]] "$OUTPUT_FOLDER is not a folder"
check_executable_exists jq
check_executable_exists cat
check_executable_exists fd
check_executable_exists sed
check_executable_exists iconv

echo "Parsing messages by $AUTHOR in $INPUT_FOLDER and producing result in $OUTPUT_FILE"
read -p "Are you sure? " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    # Delete existing output file if exists
    rm -f "$OUTPUT_FILE"
    fd . "$INPUT_FOLDER" -e json -X cat {} | \
        # Select messages by author
        jq ".messages[] | select( .sender_name == \"$AUTHOR\") | .content" | \
        # Fix encoding
        iconv -t iso-8859-1 -f utf-8 | \
        # Remove leading and trailing quotes
        sed -e 's/^"//' -e 's/"$//' | \
        # Convert "\n" to newlines
        sed 's/\\n/\'$'\n''/g' | \
        # Remove empty lines
        sed '/^[[:space:]]*$/d' > "$OUTPUT_FILE"
fi
