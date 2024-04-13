package main

import (
	"shared"
)

func main() {
	shared.InitLogger()

	Reload()

	// wait forever
	select {}
}
