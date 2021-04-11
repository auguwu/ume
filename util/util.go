package util

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"net/http"
)

func WriteJson(w http.ResponseWriter, r *http.Request, statusCode int, data interface{}) {
	r.Header.Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(statusCode)
	if err := json.NewEncoder(w).Encode(data); err != nil {
		return
	}
}

// thanks ice for this uwu
func RandomString() string {
	bytes := make([]byte, 4)
	_, _ = rand.Read(bytes)

	return fmt.Sprintf("%x", bytes)
}
