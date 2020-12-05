package main

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"

	"github.com/pkg/sftp"
	"github.com/gookit/color"
)

// Separator is file separator for current OS
const Separator = string(filepath.Separator)

// UploadExecutor upload files to server via sftp
func UploadExecutor(client *sftp.Client, srcPrefix, source, destination string) error {
	if source == "." {
		source = ""
	}

	var fullSource = srcPrefix
	if source != "" {
		fullSource = srcPrefix + "/" + source
	}

	var fullDestination = destination + source

	color.FgBlue.Println("         put to: " + fullDestination)
	color.FgBlue.Println("           from: " + fullSource)

	var fileDir, fileName = filepath.Split(fullDestination)

	var fileinfo, err = os.Stat(fullSource)
	if os.IsNotExist(err) {
		return errors.New("source `" + fullSource + "' does not exists")
	}

	var destPrefixPath = filepath.FromSlash(fullDestination)
	if len(destPrefixPath) > 0 && destPrefixPath != "./" {
		destPrefixPath += "/"
	}

	if fileinfo.IsDir() {
		if err = createDir(client, fullDestination); err != nil {
			return err
		}
		return uploadDir(client, srcPrefix, filepath.FromSlash(fullSource), destPrefixPath)
	}

	println()
	fmt.Printf("       source: %s\n", source)
	fmt.Printf("   src prefix: %s\n", srcPrefix)
	fmt.Printf("  dest prefix: %s\n", destPrefixPath)
	fmt.Printf("     file dir: %s\n", fileDir)
	fmt.Printf("    file name: %s\n", fileName)

	if err = createDir(client, fileDir); err != nil {
		return err
	}

	return uploadFile(client, srcPrefix, srcPrefix+"/"+source, fullDestination)
}

func createDir(client *sftp.Client, destination string) error {
	var path = ""
	if len(destination) > 0 && destination != "./" {
		color.FgGreen.Println("   create dir: " + destination)
		var dirs = strings.Split(filepath.FromSlash(destination), Separator)
		for _, d := range dirs {
			if len(path) == 0 {
				path = path + d
			} else {
				path = path + "/" + d
			}
			fileinfo, err := client.Stat(path)
			if err != nil && fileinfo == nil {
				if err = client.Mkdir(path); err != nil {
					return fmt.Errorf("error creation dir %s: %v", d, err)
				}
			}
		}
	}
	return nil
}

func uploadDir(client *sftp.Client, srcPrefix, source, destPrefixPath string) error {
	var stack = make([][]string, 32)
	var stackLength = 0

	var parentDir []string
	var currentDir = strings.Split(source, Separator)

	var sourcePrefixLen = len(currentDir)

	println()
	color.FgGreen.Println("  dest prefix: " + destPrefixPath)
	color.FgGreen.Println("      cur dir: " + source)

	var splittedPrefix = strings.Split(destPrefixPath, Separator)
	destPrefixPath = strings.Join(splittedPrefix, "/")

	var tarGzIndex int
	var tarGzFileOffset int
	var tarGzFileName string
	var tarBuffer *bytes.Buffer

	var gzWriter *gzip.Writer
	var tarWriter *tar.Writer

	var ret = filepath.Walk(source, func(path string, f os.FileInfo, err error) error {
		if err != nil {
			return fmt.Errorf("error walk dir%s: %v", path, err)
		}
		if (strings.Contains(path, ".git") && !strings.Contains(path, "com.git")) || strings.Contains(path, ".inventory.yaml") {
			// skip .git folder
			return nil
		}
		var splittedPath = strings.Split(filepath.FromSlash(path), Separator)

		var currentLen = len(currentDir)
		var newLen = len(splittedPath)

		var filename = destPrefixPath + strings.Join(splittedPath[sourcePrefixLen:newLen], "/")

		// fmt.Printf("     filename: %s cur-len: %v, new-len: %v\n", filename, currentLen, newLen)

		if tarGzFileName == "" {
			if f.IsDir() && strings.HasSuffix(path, ".tar.gz") {
				// enter tar.gz mode
				tarGzIndex = newLen
				tarGzFileName = filename
				color.FgGreen.Println("  >> enter tar.gz mode: `" + tarGzFileName + "`")
				tarBuffer = new(bytes.Buffer)
				gzWriter = gzip.NewWriter(tarBuffer)
				tarWriter = tar.NewWriter(gzWriter)
				tarGzFileOffset = len(splittedPath)
			}
		} else if newLen <= tarGzIndex {
			// finalize tar.gz mode
			color.FgGreen.Println("  >> leave tar.gz mode")
			tarWriter.Close()
			gzWriter.Close()
			if err := uploadTarBuffer(client, tarBuffer, tarGzFileName); err != nil {
				return err
			}
			tarGzFileName = ""
		}

		if newLen == currentLen {
			// new suffix on same prefix
		} else if newLen > currentLen {
			// new level
			stack[stackLength] = parentDir
			stackLength++
			parentDir = currentDir
		} else {
			// back to levels
			stackLength -= currentLen - newLen
			parentDir = stack[stackLength]
		}

		currentDir = splittedPath

		if tarGzFileName == "" {
			return uploadOrdinalEntry(client, f, srcPrefix, path, filename)
		}
		filename = strings.Join(splittedPath[tarGzFileOffset:newLen], "/")
		return uploadTarGzEntry(tarWriter, f, srcPrefix, path, filename)
	})

	if ret == nil && tarGzFileName != "" {
		// finalize tar.gz mode
		color.FgGreen.Println("  >> leave tar.gz mode")
		tarWriter.Close()
		gzWriter.Close()
		if err := uploadTarBuffer(client, tarBuffer, tarGzFileName); err != nil {
			return err
		}
	}

	return ret
}

func uploadOrdinalEntry(client *sftp.Client, f os.FileInfo, srcPrefix, path, filename string) error {
	// fmt.Printf("       source: %s\n", source)

	if f.IsDir() {
		if filename != "" && filename != "./" {
			_, err := client.Stat(filename)
			if err != nil {
				if os.IsNotExist(err) {
					err = createDir(client, filename)
					if err != nil {
						return err
					}
				} else {
					return fmt.Errorf("error load remote dir info %s: %v", filename, err)
				}
			}
		}
	} else {
		if err := uploadFile(client, srcPrefix, path, filename); err != nil {
			return err
		}
	}
	return nil
}

func uploadTarGzEntry(tarWriter *tar.Writer, f os.FileInfo, srcPrefix, source, filename string) error {
	filename = strings.Replace(filename, "\\", "/", -1)

	if f.IsDir() {
		// skip
	} else {
		fp, err := os.Open(source)
		if err != nil {
			return fmt.Errorf("can not open source file %s: %v", source, err)
		}
		defer fp.Close()

		dat, err := ioutil.ReadAll(fp)
		if err != nil {
			return fmt.Errorf("can not read file %s: %v", source, err)
		}
		datalen := int64(len(dat))
		if datalen != f.Size() {
			return fmt.Errorf("size of readed file: %d <> fs file size: %d", datalen, f.Size())
		}

		header := &tar.Header{
			Name:    filename,
			Size:    datalen,
			ModTime: f.ModTime(),
		}
		if err := tarWriter.WriteHeader(header); err != nil {
			return fmt.Errorf("could not write into tarball header for file `%s`: %v", filename, err)
		}
		if _, err := tarWriter.Write(dat); err != nil {
			return fmt.Errorf("could not write into tarball for file `%s`: %v", filename, err)
		}
	}

	return nil
}

func uploadFile(client *sftp.Client, srcPrefix, source, filename string) error {
	fp, err := os.Open(source)
	if err != nil {
		return fmt.Errorf("can not open source file %s: %v", source, err)
	}
	defer fp.Close()

	var executable = false
	// TODO: executable tttrabute for source file attributes
	if strings.HasSuffix(filename, "py") || strings.HasSuffix(filename, "sh") {
		executable = true
	} else if strings.HasSuffix(filename, "-x") {
		executable = true
		filename = filename[0 : len(filename)-2]
	}

	f, err := uploadWriter(client, fp, filename, source[len(srcPrefix)+1:])
	if err != nil {
		return err
	}

	if executable {
		mode := int(0764)
		f.Chmod(os.FileMode(mode))
	}

	return nil
}

func uploadTarBuffer(client *sftp.Client, tarBuffer *bytes.Buffer, filename string) error {
	if _, err := uploadWriter(client, tarBuffer, filename, filename); err != nil {
		return err
	}
	return nil
}

func uploadWriter(client *sftp.Client, reader io.Reader, filename, transFilename string) (*sftp.File, error) {
	f, err := client.Create(filename)
	if err != nil {
		return nil, fmt.Errorf("can not create destination file %s: %v", filename, err)
	}

	written, err := io.Copy(f, reader)
	if err != nil {
		return nil, fmt.Errorf("can not copy source file %s: %v", filename, err)
	}
	c := color.New(color.FgMagenta)
	c.Print("  transmitted: ")
	fmt.Printf("`%s`: ", transFilename)
	color.FgMagenta.Printf("%v bytes\n", written)

	return f, nil
}
