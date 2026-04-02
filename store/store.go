// Package store
package store

import (
	"errors"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"
)

type FilePath string
type Command string

type Config struct {
	args     []string
	database FilePath
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

	query := config.readArgs()
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

func (config *Config) readArgs() (*Query, error) {
	args := os.Args[1:]

	if len(args) < 2 {
		log.Fatal("Please specify a command and arguments")
	}

	command := os.Args[0]

	args = args[1:]

	switch command {
	case "set":
		if len(args) != 2 {
			log.Fatal("Error: 'set' requires exactly 2 arguments (key, value)")
		}

		key, value := args[0], strings.Join(args[1:], "")
		return &Query{key, value}, nil
	case "get":
		if len(args) != 1 {
			log.Fatal("Error: 'get' requires exactly a single argument (key) to get")
		}

		return &Query{key: args[0]}, nil

	default:
		log.Fatal("idk")
	}

	return nil, errors.New("Couldn't read arguments")

}

type Query struct {
	key   string
	value string
}

func (query *Query) parseQuery() string {
}
