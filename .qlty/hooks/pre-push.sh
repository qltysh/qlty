#!/bin/sh

# Install with:
#
# ln -s ../../.qlty/hooks/pre-push.sh .git/hooks/pre-push
# chmod +x .git/hooks/pre-push

qlty check --trigger pre-push --upstream-from-pre-push --no-formatters --skip-errored-plugins
