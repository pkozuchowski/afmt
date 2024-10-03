#!/bin/bash

# File containing list of repositories
REPO_LIST="repos.txt"
TARGET_DIR="repos"

# Create target directory if it doesn't exist
mkdir -p $TARGET_DIR

# Loop over each repository in repos.txt
while IFS= read -r REPO_URL; do
    # Extract repo name from URL
    REPO_NAME=$(basename -s .git "$REPO_URL")

    # Clone the repository
    echo "Cloning $REPO_URL into $TARGET_DIR/$REPO_NAME"
    git clone "$REPO_URL" "$TARGET_DIR/$REPO_NAME"
done < "$REPO_LIST"