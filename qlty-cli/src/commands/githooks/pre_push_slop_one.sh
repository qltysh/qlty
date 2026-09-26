#!/bin/sh
# Git passes the refs being pushed on stdin, and both steps need them.
input=$(cat)

printf '%s\n' "$input" | qlty check \
	--trigger pre-push \
	--upstream-from-pre-push \
	--no-formatters \
	--skip-errored-plugins || exit $?

printf '%s\n' "$input" | qlty slop-one --trigger pre-push --upstream-from-pre-push
