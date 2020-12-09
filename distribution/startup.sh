#!/bin/bash

export LD_LIBRARY_PATH="/opt/foundation/instantclient/"

## options -qq and others for suppress output
apt-get -qq update && apt-get install -qq -o=Dpkg::Use-Pty=0 libaio1 libaio-dev -y \
        && rm -rf /var/lib/apt/lists/*

./bin/server
