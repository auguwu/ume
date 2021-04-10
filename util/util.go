package util

import (
	"encoding/json"
	"net/http"
)

func WriteJson(w http.ResponseWriter, r *http.Request, statusCode int, data interface{}) {
	r.Header.Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(statusCode)
	if err := json.NewEncoder(w).Encode(data); err != nil {
		return
	}
}
