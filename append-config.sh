#!/bin/bash

# Switch to script's directory, letting it be called from any folder.
cd $(dirname $0)

CARGO_METADATA=$(cargo metadata --format-version 1 --manifest-path bevy/Cargo.toml --no-deps)

# Capture the name, excluding the directory and extension.
REGEX='^config\/([a-z_]+)\.toml$'

# Iterate over all files in the config folder.
for filename in config/*.toml; do
    if [[ $filename =~ $REGEX ]]; then
        # Extract the name of the package from the filename.
        NAME=${BASH_REMATCH[1]}

        # Find the path to the manifest.
        MANIFEST=$(echo $CARGO_METADATA | jq --raw-output ".packages[] | select(.name == \"$NAME\") | .manifest_path")

        echo Appending $NAME config from $filename to $MANIFEST.

        # Append the config.
        cat $filename >> $MANIFEST
    else
        echo Error extracting package name from $filename.
    fi
done
