#!/bin/bash

DEBUG="${DEBUG:-true}"

DEBUG=$DEBUG npx nodemon --ext go --exec go run .
