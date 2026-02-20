#!/usr/bin/env bash

VERSION=$1
echo "Preparing release for v${VERSION}"

echo "Creating GIT tag for release version"
git tag --sign -a v${VERSION}

source .ci/local/generate_pkgbuild.sh

mkdir -p target/release/gh_release
echo "Creating release tarball"
TAR_FILE="target/release/gh_release/${REL_NAME}.tar.gz"
git -c tar.tar.gz.command='gzip -cn' archive --format=tar.gz --prefix="${REL_NAME}/" -o "${TAR_FILE}" "v${VERSION}"

echo "Signing release tarball"
TAR_SIG_FILE="target/release/gh_release/${REL_NAME}.tar.gz.sig"
gpg --detach-sign --output "${TAR_SIG_FILE}" $TAR_FILE

echo "Updating PKGBUILD"
TAR_HASH=$(sha256sum ${TAR_FILE} | awk '{print $1}')
TAR_SIG_HASH=$(sha256sum ${TAR_SIG_FILE} | awk '{print $1}')
generate_pkgbuild remote ${TAR_HASH} ${TAR_SIG_HASH}