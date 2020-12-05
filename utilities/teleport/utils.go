package main

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"runtime"

	"github.com/gookit/color"
)

// Variable info about path prefix
type Variable struct {
	Description string
	Path        string
}

// Dictionary is a map for content variables
type Dictionary map[string]Variable

// TeleportDir get path to teleport executable
func TeleportDir() string {
	executable, _ := os.Executable()
	return filepath.Dir(executable)
}

// UserHomeDir get path to User Home dir
func UserHomeDir() string {
	if runtime.GOOS == "windows" {
		home := os.Getenv("HOMEDRIVE") + os.Getenv("HOMEPATH")
		if home == "" {
			home = os.Getenv("USERPROFILE")
		}
		return home
	}
	return os.Getenv("HOME")
}

func expandTemplate(template string, regex *regexp.Regexp, variables Dictionary) string {
	return regex.ReplaceAllStringFunc(template, func(s string) string {
		// str := s[1:len(s)]
		variable, ok := variables[s]
		if ok {
			return variable.Path
		}
		return s[1:len(s)]
	})
}

func usage(inventory Inventory, variables Dictionary) {
	println()
	color.FgLightBlue.Println("USAGE")
	fmt.Print("args: ")
	color.FgLightBlue.Println("[server] [source]")

	c := color.New(color.FgLightBlue)

	println("servers in inventory:")

	for key, server := range inventory.Servers {
		c.Print("   " + key)
		fmt.Printf(": %s [%s]\n", server.URI, server.Description)
	}

	println()
	println("content path templates:")

	for key, value := range variables {
		c.Print("   " + key)
		fmt.Printf(": %s [%s]\n", value.Description, value.Path)
	}

	os.Exit(0)
}
