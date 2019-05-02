#!/usr/bin/env bash

cd "$(dirname $0)"

echo "===== commented out C code ====="

tot=0
for f in kern/src/*.rs *.S; do
  cnt=$(grep -E '////' $f | wc -l)
  if [[ cnt -gt 0 ]]; then
    tot=$((tot + cnt))
    printf "%12s %i\n" $(basename $f) $cnt
  fi
done
printf "%-12s %i\n" "Total:" $tot
