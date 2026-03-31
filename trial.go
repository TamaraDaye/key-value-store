package main

import (
	"fmt"
	// "log"
	"os"
)

func main() {
	args := os.Args

	fmt.Println(args[1:])

	for _, word := range args[1:] {
		fmt.Println(word)
	}
}
