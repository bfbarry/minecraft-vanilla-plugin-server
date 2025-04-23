package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"strings"
	"time"
)

func main() {
	logFile := "/home/barry_brian_f/server/logs/latest.log" 
	items, err := readItemsFromFile("/home/barry_brian_f/server/sidecar/items.txt")
	if err != nil {
		fmt.Println("Error reading items:", err)
		return
	}

	// Regular expression to match chat lines
	re := regexp.MustCompile(`\[.*?\]: <(.*?)> \.kit`)

	file, err := os.Open(logFile)
	if err != nil {
		fmt.Printf("Error opening log file: %v\n", err)
		return
	}
	defer file.Close()

	// Seek to the end of file
	file.Seek(0, os.SEEK_END)

	reader := bufio.NewReader(file)

	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			time.Sleep(500 * time.Millisecond) // Wait before retrying
			continue
		}

		// Check if the line matches our pattern
		if matches := re.FindStringSubmatch(line); matches != nil {
			player := matches[1]
			for _, item := range items {
				cmd := fmt.Sprintf("give %s %s", player, item)
				sendCommandToTmux("mcserver", cmd)
			}
		}
	}
}

func readItemsFromFile(filename string) ([]string, error) {
	var items []string

	file, err := os.Open(filename)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		item := strings.TrimSpace(scanner.Text())
		if item != "" {
			items = append(items, item)
		}
	}

	if err := scanner.Err(); err != nil {
		return nil, err
	}

	return items, nil
}

func sendCommandToTmux(session string, command string) {
	// Construct the tmux send-keys command
	cmd := exec.Command("tmux", "send-keys", "-t", session, command, "C-m")
	err := cmd.Run()
	if err != nil {
		fmt.Printf("Error sending command to tmux: %v\n", err)
	}
}

