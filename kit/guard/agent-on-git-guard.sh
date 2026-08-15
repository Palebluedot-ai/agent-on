#!/bin/sh
# Polyglot compatibility path: v0.6-era Codex hooks invoked this .sh with
# python3; v0.7+ hooks may invoke it with bash. Both delegate to one guard.
""":"
HERE="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec bash "${HERE}/agent-on-git-guard" "$@"
":"""
import os
import sys

target = os.path.join(os.path.dirname(os.path.realpath(__file__)), "agent-on-git-guard")
os.execvp("bash", ["bash", target, *sys.argv[1:]])
