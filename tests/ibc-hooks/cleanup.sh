#!/bin/sh
exec find . -type d -maxdepth 1 -regex '\./[0-9]*' -exec rm -rf {} '+'
