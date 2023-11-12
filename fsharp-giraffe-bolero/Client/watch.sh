#!/bin/bash

ASPNETCORE_URLS=http://localhost:8000 \
ASPNETCORE_ENVIRONMENT=Development \
DOTNET_ROLL_FORWARD=LatestMajor \
dotnet watch
