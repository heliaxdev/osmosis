#!/bin/sh
export WASM_SUFFIX="-aarch64.wasm"
exec go test -mod=readonly -v -test.v -rapid.v -run ^TestIBCHooksTestSuite/TestBingBong
