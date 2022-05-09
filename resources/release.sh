#!/bin/bash
RELEASE_TAG=v0.8.3
cd ../releases

rm -rf $RELEASE_TAG.tar
tar -cf $RELEASE_TAG.tar ../builds
# tar –xf $RELEASE_TAG.tar $RELEASsE_TAGs

