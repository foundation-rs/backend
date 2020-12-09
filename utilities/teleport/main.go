package main

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"regexp"

	"github.com/gookit/color"
	"github.com/pkg/sftp"
	"golang.org/x/crypto/ssh"
)

func main() {
	executablePath := TeleportDir()
	homePath := UserHomeDir()
	fmt.Println("executable path: " + executablePath)
	fmt.Println("      home path: " + homePath)

	inventory := LoadInventory(homePath + Separator + ".inventory.yaml")
	// inventory := LoadInventory(executablePath + Separator + ".inventory.yaml")

	var variables = map[string]Variable{
		"@HOME": {Description: "User Home", Path: homePath},
		"@DESK": {Description: "Desktop", Path: homePath + Separator + "Desktop"},
		"@DOCS": {Description: "Documents", Path: homePath + Separator + "Documents"},
	}

	if len(os.Args) < 3 {
		usage(inventory, variables)
	}

	server := os.Args[1]
	source := os.Args[2]

	serverConfig, ok := inventory.Servers[server]
	if !ok {
		color.Red.Printf("not found server `%s` in inventory\n", server)
	}

	sshConfig := &ssh.ClientConfig{
		User:            serverConfig.User,
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Auth: []ssh.AuthMethod{
			ssh.Password(serverConfig.Password),
		},
	}

	Upload(&serverConfig, sshConfig, variables, inventory.Content, source, homePath, executablePath)

	color.Blue.Println("OK")
}

// Connect create sftp session for server in serverConfig
func Connect(serverConfig *ServerConfig, sshConfig *ssh.ClientConfig) (connection *ssh.Client, sftpClient *sftp.Client) {
	url := serverConfig.URI + ":22"
	connection, err := ssh.Dial("tcp", url, sshConfig)
	if err != nil {
		log.Fatalf("Failed to dial to server `%s`: %s", url, err)
	}
	sftpClient, err = sftp.NewClient(connection)
	if err != nil {
		log.Fatalf("unable to start sftp subsystem: `%s`: %s", url, err)
	}
	return connection, sftpClient
}

// Upload source file|dir from contentPath to server
func Upload(
	serverConfig *ServerConfig, sshConfig *ssh.ClientConfig,
	variables Dictionary,
	contentPath, source, homePath, executablePath string) {
	connection, sftpClient := Connect(serverConfig, sshConfig)
	defer connection.Close()
	defer sftpClient.Close()

	regex, _ := regexp.Compile(`@[a-zA-Z]+`)

	contentPath = filepath.Join(expandTemplate(contentPath, regex, variables), serverConfig.PathPrefix)

	var destination = "./"
	if serverConfig.DestPrefix != "" {
		destination = serverConfig.DestPrefix + "/"
	}

	if err := UploadExecutor(sftpClient, contentPath, source, destination); err != nil {
		color.Red.Printf("%s\n", err)
	}
}
