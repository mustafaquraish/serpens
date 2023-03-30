#!/bin/bash

set -xe

cargo run -- -c $1 > $1.cc
g++ -std=c++20 -ggdb3 $1.cc runtime/*.cc
./a.out