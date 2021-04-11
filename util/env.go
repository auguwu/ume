package util

import "os"

func Auth() string {
	return os.Getenv("AUTH")
}
