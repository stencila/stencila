#!/usr/bin/env bash

# Upload files to a GitHub release
#
#    upload-release.sh <path> <asset> <triple>.<format>
#
# <path> is the local path to the binary, <asset> is the asset name, <target> is
# the target triple and <format> is the archive format (i.e. `zip` or `tar.gz`)
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
TRIPLE_FORMAT=$3

# If the extension of the file is the same as the desired <triple>.<format>
# (i.e. already an archive) then just upload the file, otherwise create an archive
# of the appropriate format
if [ "${FILE_PATH##*.}" == "${TRIPLE_FORMAT##*.}" ]; then
    UPLOAD_PATH=$FILE_PATH
else
    ARCHIVE_FORMAT="${TRIPLE_FORMAT#*.}"
    UPLOAD_PATH="$FILE_PATH.$ARCHIVE_FORMAT"
    echo "Will create archive $UPLOAD_PATH"
    if [ $ARCHIVE_FORMAT == "zip" ]; then
        if [ -x "$(command -v zip)" ]; then
            (cd $(dirname $FILE_PATH) && zip -r - $(basename $FILE_PATH)) > $UPLOAD_PATH
        else
            7z.exe a -tzip -bb3 -mx=5 $UPLOAD_PATH $FILE_PATH
        fi
    elif [ $ARCHIVE_FORMAT == "tar.gz" ]; then
        tar -C $(dirname $FILE_PATH) -czvf $UPLOAD_PATH $(basename $FILE_PATH)
    fi
fi

TAG=$(git describe --tags --abbrev=0)
echo "Will upload for tag $TAG"

RELEASE_ID=$(curl -s "https://api.github.com/repos/stencila/stencila/releases/tags/$TAG" | grep -m 1 "id.:" | cut -c9-16)
echo "Will upload to release $RELEASE_ID"

DOWNLOAD_NAME="$ASSET_NAME-$TAG-$TRIPLE_FORMAT"
UPLOAD_URL="https://uploads.github.com/repos/stencila/stencila/releases/$RELEASE_ID/assets?name=$DOWNLOAD_NAME"
echo "Will upload $UPLOAD_PATH to $UPLOAD_URL"

AUTH_HEADER="Authorization: token $GITHUB_TOKEN"
curl -H "$AUTH_HEADER" -H "Content-Type: application/octet-stream" -o /dev/null --data-binary @"$UPLOAD_PATH" "$UPLOAD_URL"
echo "Upload complete"
