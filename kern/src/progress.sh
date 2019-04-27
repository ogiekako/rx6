#!/usr/bin/env bash

cd "$(dirname $0)"

echo "===== commented out C code ====="

tot=0
for f in *rs; do
  cnt=$(grep -E '////' $f | wc -l)
  tot=$((tot + cnt))
  printf "%12s %i\n" $f $cnt
done
printf "%-12s %i\n" "Total:" $tot
