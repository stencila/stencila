#!/usr/bin/env bash

# Upload files to a GitHub release
#
#    upload-release.sh ./target/release/stencila "Linux%20CLI"
#
# Expects the $GITHUB_TOKEN to be set and for the release for the current tag to have already
# been created.

set -e

FILE_PATH=$2
FILE_LABEL=$3

AUTH_HEADER="Authorization: token $GITHUB_TOKEN"
TAG=$(git describe --tags --abbrev=0)
RELEASE_ID=$(curl -s -H "$AUTH_HEADER" "https://api.github.com/repos/stencila/stencila/releases/tags/$TAG" | grep -oP '(?<=^  "id": )\d+')
FILE_NAME=$(basename $FILE_PATH)
UPLOAD_URL="https://uploads.github.com/repos/stencila/stencila/releases/$RELEASE_ID/assets?name=$FILE_NAME&label=$FILE_LABEL"
curl -H "$AUTH_HEADER" -H "Content-Type: application/octet-stream" -o /dev/null --data-binary @"$FILE_PATH" "$UPLOAD_URL"
echo "Uploaded $FILE_PATH to $UPLOAD_URL"
