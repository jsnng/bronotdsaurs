#! /bin/zsh
timeout 580 cargo kani -Z function-contracts -j $(sysctl -n hw.perflevel0.physicalcpu)