#!/bin/bash

# --index-file '${env.GIT_INDEX_FILE}'
# qlty fmt --trigger pre-commit --upstream=HEAD

echo $GIT_INDEX_FILE

# ln -s ../../.qlty/hooks/pre-commit.sh .git/hooks/pre-commit
# chmod +x .git/hooks/pre-commit
# ./.git/hooks/pre-commit