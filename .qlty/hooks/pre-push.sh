#!/bin/bash

# Install with:
#
# ln -s ../../.qlty/hooks/pre-push.sh .git/hooks/pre-push
# chmod +x .git/hooks/pre-push

while IFS= read -r line; do
	echo "$line"
done
