#!/bin/bash
curl https://qlty.sh | sh
source /root/.profile
qlty --help
qlty install || echo "qlty install failed"
