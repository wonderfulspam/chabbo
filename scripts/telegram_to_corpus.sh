#!/bin/bash

## TELEGRAM TO CORPUS ##
# Filters a JSON chat export by author and produces a corpus for use in Chabbo

DIR="$(dirname "$(realpath "$0")")"
source "${DIR}/helpers.sh"

INPUT_FILE=$1
OUTPUT_FOLDER=$2
AUTHOR=$3
OUTPUT_FILE="${OUTPUT_FOLDER}/${AUTHOR// /_}.txt" # Convert spaces to underscores
OUTPUT_FILE=${OUTPUT_FILE//\/\//\/} # Remove double slashes

# Sanity checks
test_or_exit [[ $# -eq 3 ]] "Must provide three input arguments: input file, output folder and author"
test_or_exit [[ -f "$INPUT_FILE" ]] "No file at $INPUT_FILE"
test_or_exit [[ -d "$OUTPUT_FOLDER" ]] "$OUTPUT_FOLDER is not a folder"
check_executable_exists jq
check_executable_exists cat
check_executable_exists sed

echo "Parsing messages by $AUTHOR in $INPUT_FILE and producing result in $OUTPUT_FILE"
read -p "Are you sure? " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    cat $INPUT_FILE | \
        # Select messages by author
        # Filter by plain text messages
        jq ".messages[] | select( .from == \"$AUTHOR\") | \
        .text_entities[] | select( .type = \"plain\") | .text" | \
        # Remove leading and trailing quotes
        sed -e 's/^"//' -e 's/"$//' | \
        # Convert "\n" to newlines
        sed 's/\\n/\'$'\n''/g' | \
        # Remove empty lines
        sed '/^[[:space:]]*$/d' > $OUTPUT_FILE
fi
