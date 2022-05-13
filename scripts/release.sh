#!/bin/bash
RELEASE_TAG=v0.8.4
cd ..
rm -rf releases/$RELEASE_TAG.tar
tar -czf releases/$RELEASE_TAG.tar.gz builds

