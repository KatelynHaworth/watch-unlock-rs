#!/usr/bin/env bash

LATEST_TAG=$(git describe --tags --abbrev=0)

export PKG_NAME=$(basename $PWD)
export PKG_VER=${LATEST_TAG#"v"}
export PKG_REL=$(git rev-list ${LATEST_TAG}.. --count)
export REL_NAME="${PKG_NAME}-${PKG_VER}"

function generate_pkgbuild() {
    PKG_SOURCE=""
    PKG_SOURCE_HASH=""
    PKG_SOURCE_SIG_HASH=""

    case $1 in
      "remote")
        if [[ -z "$2" ]] || [[ -z "$3" ]]; then
          echo "Mode 'remote' requires two parameters: [sha256_source_hash] [sha256_signature_hash]"
          exit 1
        fi

        PKG_SOURCE='${url}/releases/download/v$pkgver/$pkgname-$pkgver.tar.gz{,.sig}'
        PKG_SOURCE_HASH=$2
        PKG_SOURCE_SIG_HASH=$3
        PKG_FILE=$4
        ;;

      "local")
        PKG_SOURCE='$pkgname-$pkgver.tar.gz'
        PKG_SOURCE_HASH="SKIP"
        PKG_FILE=$2
      ;;

      *)
        echo "Unsupported PKGBUILD generation mode"
        exit 1
        ;;
    esac

    if [[ -z "$PKG_FILE" ]]; then
      PKG_FILE="PKGBUILD"
    fi

    awk "{
      sub(/{{PKG_NAME}}/,\"$PKG_NAME\");
      sub(/{{PKG_VER}}/,\"$PKG_VER\");
      sub(/{{PKG_REL}}/,\"$PKG_REL\");
      sub(/{{PKG_SOURCE}}/,\"$PKG_SOURCE\");
      sub(/{{PKG_SOURCE_HASH}}/,\"$PKG_SOURCE_HASH\");
      sub(/{{PKG_SOURCE_SIG_HASH}}/,\"$PKG_SOURCE_SIG_HASH\");
    }1" .ci/local/PKGBUILD.source > $PKG_FILE
}

