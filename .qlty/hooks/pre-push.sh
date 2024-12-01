#!/bin/bash

# Install with:
#
# ln -s ../../.qlty/hooks/pre-push.sh .git/hooks/pre-push
# chmod +x .git/hooks/pre-push

input=$(cat)
echo $input
