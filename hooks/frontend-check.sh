#!/bin/bash
set -e
cd frontend
npm audit --ws
knip
osv-scanner scan source -r .
