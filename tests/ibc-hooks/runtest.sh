#!/bin/sh
exec go test -mod=readonly -v -test.v -rapid.v -run ^TestIBCHooksTestSuite/TestBingBong -o bongus
