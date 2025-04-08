#!/bin/bash

set -xe

SCAN_DIR=/tmp/scan

if [ ! -e /etc/mirakc/config.yml ]; then
    mkdir -p ${SCAN_DIR}
    pushd ${SCAN_DIR}
        isdb-scanner --exclude-pay-tv
	cat - <<EOT >> scanned/mirakc/config.yml
recording:
  basedir: /recorded
  records-dir: /recorded/records
EOT
	mv scanned/mirakc/config.yml /etc/mirakc/config.yml
    popd
    rm -rf ${SCAN_DIR}
fi

if [ ! -d /recorded/records ]; then
    mkdir -p /recorded/records
fi

exec /usr/local/bin/mirakc
