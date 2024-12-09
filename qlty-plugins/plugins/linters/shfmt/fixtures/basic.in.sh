#!/bin/bash
num=`awk '{print $1}' $i`
if (( $num >= 1000 )); then  # potential CPU hogs?
fi
