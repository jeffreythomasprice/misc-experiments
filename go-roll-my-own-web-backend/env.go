package main

import (
	"os"
	"strings"
)

func isDebug() bool {
	return getBoolEnvironmentVariable("DEBUG", false)
}

func getBoolEnvironmentVariable(key string, def bool) bool {
	value, exists := os.LookupEnv(key)
	if !exists {
		return def
	}
	return strings.ToLower(strings.TrimSpace(value)) == "true"
}
