#!/bin/bash

cd "$(dirname "$0")"

rm -rf certs
mkdir -p certs
cd certs

openssl genrsa -out jwt-private.pem 4096
openssl rsa -in jwt-private.pem -outform PEM -pubout -out jwt-public.pem
