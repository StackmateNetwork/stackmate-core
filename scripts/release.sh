#!/bin/bash
RELEASE_TAG=v0.9.0

if (( $EUID == 0 )); then
    REPO="/stackmate-core"
else
    REPO="$HOME/StackmateNetwork/stackmate-core"
fi


cd $REPO
rm -rf $RELEASE_TAG.tar
tar -czf $RELEASE_TAG.tar.gz builds

