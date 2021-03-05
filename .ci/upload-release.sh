#!/usr/bin/env bash

# Upload files to a GitHub release
#
#    upload-release.sh <path> <asset> <triple> <format> <skip archiving>
#
# <path> is the local path to the binary, <asset> is the asset name, <target> is
# the targe triple and <format> is the archive format (i.e. `zip` or `tar.gz`)
# See https://github.com/japaric/trust/releases for an example
# of the download names we are targetting for compatability with `self_update` crate.
#
#    upload-release.sh target/release/stencila stencila x86_64-unknown-linux-gnu.tar.gz
#    upload-release.sh target/release/stencila.exe stencila x86_64-pc-windows-msvc.zip
#
# Expects the $GITHUB_TOKEN to be set and for the release for the current tag to have already
# been created.

set -e

FILE_PATH=$1
ASSET_NAME=$2
TARGET_TRIPLE=$3
ARCHIVE_FORMAT=$4
SKIP_ARCHIVE=$5

ARCHIVE_PATH="$FILE_PATH.$ARCHIVE_FORMAT"
if [ -z $SKIP_ARCHIVE ]; then
    if [ $ARCHIVE_FORMAT == "zip" ]; then
        (cd $(dirname $FILE_PATH) && zip -r - $(basename $FILE_PATH)) > $ARCHIVE_PATH
    elif [ $ARCHIVE_FORMAT == "tar.gz" ]; then
        tar -C $(dirname $FILE_PATH) -czvf $ARCHIVE_PATH $(basename $FILE_PATH)
    fi
fi

TAG=$(git describe --tags --abbrev=0)
echo "Will upload for tag $TAG"

AUTH_HEADER="Authorization: token $GITHUB_TOKEN"
RELEASE_ID=$(curl -s -H "$AUTH_HEADER" "https://api.github.com/repos/stencila/stencila/releases/tags/$TAG" | grep -m 1 "id.:" | cut -c9-16)
echo "Will upload to release $RELEASE_ID"

DOWNLOAD_NAME="$ASSET_NAME-$TAG-$TARGET_TRIPLE.$ARCHIVE_FORMAT"
UPLOAD_URL="https://uploads.github.com/repos/stencila/stencila/releases/$RELEASE_ID/assets?name=$DOWNLOAD_NAME"
echo "Will upload $ARCHIVE_PATH to $UPLOAD_URL"

curl -H "$AUTH_HEADER" -H "Content-Type: application/octet-stream" -o /dev/null --data-binary @"$ARCHIVE_PATH" "$UPLOAD_URL"
echo "Upload complete"
