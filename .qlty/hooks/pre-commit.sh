#!/bin/sh

# Install with:
#
# ln -s ../../.qlty/hooks/pre-commit.sh .git/hooks/pre-commit
# chmod +x .git/hooks/pre-commit

qlty fmt --trigger pre-commit --index-file="$GIT_INDEX_FILE"
