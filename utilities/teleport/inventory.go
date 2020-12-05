package main

import (
	"io/ioutil"
	"log"

	yaml "gopkg.in/yaml.v2"
)

// Inventory id dictionary of remote servers
type Inventory struct {
	Content string                  `yaml:"content"`
	Servers map[string]ServerConfig `yaml:"servers"`
}

// ServerConfig is connection info for remote server
type ServerConfig struct {
	URI         string `yaml:"uri"`
	User        string `yaml:"user"`
	Password    string `yaml:"password"`
	Description string `yaml:"description"`
	PathPrefix  string `yaml:"path-prefix"`
}

// LoadInventory load inventory from specified filepath
func LoadInventory(path string) Inventory {
	file, err := ioutil.ReadFile(path)
	if err != nil {
		log.Fatalf("%s get err #%v", path, err)
	}
	var inventory = Inventory{}

	if err = yaml.Unmarshal(file, &inventory); err != nil {
		log.Fatalf("Unmarshal: %v", err)
	}

	return inventory
}
