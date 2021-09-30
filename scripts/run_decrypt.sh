#!/bin/bash

QUAIL=/home/ubuntu/quail
HONEYCRISP=${QUAIL}/lib/honeycrisp
SCALE_MAMBA=/root/SCALE-MAMBA

[[ $UID = 0 ]] || exec sudo $0 "$@"

# update scripts first
bash ${QUAIL}/scripts/update_honeycrisp.sh

cd ${SCALE_MAMBA}
bash ${SCALE_MAMBA}/decrypt.sh $1

