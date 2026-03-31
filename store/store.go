// Package store
package store

import (
	"errors"
	"log"
	"os"
	"path/filepath"
)

type FilePath string
type Command string

const (
	get Command = "get"
	set Command = "set"
)

type Config struct {
	args     []string
	database FilePath
	command  Command
}

func (config *Config) init() {
	if !config.checkDB() {
		rootdir, err := os.Getwd()
		if err != nil {
			log.Fatal("error encountered", err)
		}
		path := filepath.Join(rootdir, "database", "database1.bin")

		_, err = os.Create(path)

		if err != nil {
			log.Fatal("error encountered", err)
		}

		config.database = FilePath(path)
	}
}

func (config *Config) checkDB() bool {
	_, err := os.Stat(string(config.database))

	if err == nil {
		return true
	}

	if errors.Is(err, os.ErrNotExist) {
		return false
	}

	return false
}

func (config *Config) readArgs() (string, string) {
	args := os.Args[1:]

	if len(args) == 0 {
		log.Fatal("Please specify a command")
	}

	switch args[0] {
	case "get":
		config.command = get
	case "set":
		config.command = set
	}

}
