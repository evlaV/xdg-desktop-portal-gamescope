#!/bin/bash

if [ "$1" = "version" ]; then
    exit 0
elif [ "$1" = "screenshot" ]; then
    touch "$2"
    exit 0
else
    exit 1
fi
