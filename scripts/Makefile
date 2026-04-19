# Compatibility wrapper for Ubuntu/Unix make entrypoint.
# Canonical implementation lives in scripts/ubuntu/Makefile.

_THIS_MK := $(lastword $(MAKEFILE_LIST))
_THIS_DIR := $(dir $(abspath $(_THIS_MK)))

include $(_THIS_DIR)ubuntu/Makefile
