@echo off
set GOARCH=amd64
set GOOS=windows

go build
move teleport.exe ../../