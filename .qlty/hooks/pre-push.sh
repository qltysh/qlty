#!/bin/sh
qlty check --trigger pre-push --upstream-from-pre-push --no-formatters --skip-errored-plugins
