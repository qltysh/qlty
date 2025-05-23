#!/bin/sh
qlty check \
	--trigger pre-push \
	--level=low \
	--upstream-from-pre-push \
	--no-formatters \
	--skip-errored-plugins
