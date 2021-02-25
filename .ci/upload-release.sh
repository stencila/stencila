#!/usr/bin/env bash

# Upload files to a GitHub release
#
#    upload-release.sh <path> <asset> <triple> <label>
#
# <path> is the local path to the binary, <asset> is (usually) the binary name, <triple> is the target triple
# (see https://github.com/japaric/trust#supported-targets) and <label> is the label for the
# file on the GitHub release page.
#
#    upload-release.sh ./target/release/stencila stencila x86_64-unknown-linux-gnu Linux%20CLI
#
# Expects the $GITHUB_TOKEN to be set and for the release for the current tag to have already
# been created.

set -e

FILE_PATH=$1
ASSET_NAME=$2
TARGET_TRIPLE=$3
DOWNLOAD_LABEL=$4

AUTH_HEADER="Authorization: token $GITHUB_TOKEN"
TAG=$(git describe --tags --abbrev=0)
RELEASE_ID=$(curl -s -H "$AUTH_HEADER" "https://api.github.com/repos/stencila/stencila/releases/tags/$TAG" | sed -n 's/^  "id": \(\S*\),$/\1/p')
DOWNLOAD_NAME="$ASSET_NAME-$TAG-$TARGET_TRIPLE"
UPLOAD_URL="https://uploads.github.com/repos/stencila/stencila/releases/$RELEASE_ID/assets?name=$DOWNLOAD_NAME&label=$DOWNLOAD_LABEL"
curl -H "$AUTH_HEADER" -H "Content-Type: application/octet-stream" -o /dev/null --data-binary @"$FILE_PATH" "$UPLOAD_URL"
echo "Uploaded $FILE_PATH to $UPLOAD_URL"
