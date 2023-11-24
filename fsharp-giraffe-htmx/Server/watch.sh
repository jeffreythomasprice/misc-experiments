#!/bin/bash

ASPNETCORE_ENVIRONMENT=Development \
DOTNET_ROLL_FORWARD=LatestMajor \
dotnet watch
