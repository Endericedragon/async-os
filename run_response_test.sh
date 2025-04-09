#!/bin/bash
for i in $(seq 1 8)
do
    # sh run_asyncos.sh --log error > neg_log.ansi
    sh run_asyncos.sh > neg_log.ansi
    python3.13 analyze_neg_log.py "res$i.txt"
done