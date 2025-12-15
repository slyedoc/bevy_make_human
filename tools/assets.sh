#!/bin/bash

# Script to analyze asset files in a directory.

DEDUPE=false
ASSETS_DIR="./assets"

while [[ $# -gt 0 ]]; do
  case $1 in
    -d|--dedupe) DEDUPE=true; shift ;;
    *) ASSETS_DIR="$1"; shift ;;
  esac
done

if $DEDUPE; then
  echo "Ext      Count    Size MB |  Unique  Dups   Dup%   Savings MB"
  echo "--------------------------------------------------------------"
  find "$ASSETS_DIR" -type f | sed 's/.*\.//' | sort | uniq | while read ext; do
    find "$ASSETS_DIR" -type f -name "*.$ext" -exec sh -c 'md5sum "$1" && stat -c%s "$1"' _ {} \; 2>/dev/null \
      | paste - - \
      | awk -v ext="$ext" '
        {
          hash=$1; size=$NF
          count[hash]++
          filesize[hash]=size
          total_size+=size
        }
        END {
          dup_size=0; dups=0; unique=length(count)
          for(h in count) if(count[h]>1) {dups+=count[h]-1; dup_size+=filesize[h]*(count[h]-1)}
          total=unique+dups
          if(total>0) printf "%6s %7d %10.2f | %7d %5d %5.1f%% %10.2f\n", ext, total, total_size/1024/1024, unique, dups, (dups/total)*100, dup_size/1024/1024
        }'
  done | sort -t'|' -k1 -rn
else
  echo "Count  Ext       Size MB"
  echo "------------------------"
  find "$ASSETS_DIR" -type f -printf '%s %f\n' | awk -F. '{ext=$NF; size=$1; count[ext]++; total[ext]+=size} END {for(e in count) printf "%5d  %-8s %12.2f\n", count[e], e, total[e]/1024/1024}' | sort -k3 -rn
fi
