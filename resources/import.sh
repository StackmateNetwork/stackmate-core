#!/bin/bash

SERVER=debian@scb
SERVER_DIRECTORY=/home/debian/stackmate-core/builds

rm -rf ../builds

scp -r "$SERVER:$SERVER_DIRECTORY" ../

# tar -czvf releases.tar ../builds