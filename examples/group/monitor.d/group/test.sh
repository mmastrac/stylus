#!/bin/bash
set -xeuf -o pipefail
echo '@@STYLUS@@ group.port-0.status.status="yellow"'
echo '@@STYLUS@@ group.port-1.status.status="green"'
echo '@@STYLUS@@ group.port-2.status.status="yellow"'
echo '@@STYLUS@@ group.port-3.status.status="green"'
echo '@@STYLUS@@ group.port-4.status.status="green"'
echo '@@STYLUS@@ group.port-5.status.status="yellow"'
echo '@@STYLUS@@ group.port-6.status.status="yellow"'
echo '@@STYLUS@@ group.port-7.status.status="red"'
