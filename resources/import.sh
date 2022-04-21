#!/bin/bash

SERVER=debian@scb
SERVER_DIRECTORY=/home/debian/stackmate-core/builds
LOCAL_DIRECTORY=../builds

rm -rf $LOCAL_DIRECTORY
mkdir -p $LOCAL_DIRECTORY

scp -r "$SERVER:$SERVER_DIRECTORY" "$LOCAL_DIRECTORY"
