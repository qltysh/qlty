#!/bin/sh
qlty fmt --trigger pre-commit --index-file="$GIT_INDEX_FILE"
