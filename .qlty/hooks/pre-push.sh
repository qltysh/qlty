#!/bin/bash
export TERM=xterm

# Check for dirty files
if [ -n "$(git status --porcelain)" ]; then
	echo "Error: There are dirty files in the Git working directory."
	exit 1
fi

qlty fmt

# Check for dirty files after qlty fmt
if [ -n "$(git status --porcelain)" ]; then
	git add .
	git commit -m "qlty fmt"

	echo "NOTE: qlty fmt modified files. Please re-run git push."
	exit 1
fi

exit 0
