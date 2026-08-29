#!/bin/bash
set -e
cd frontend
npm audit --ws
depcheck --ignore-bin-package
osv-scanner scan source -r .
