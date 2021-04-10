package main

import (
	"floofy.dev/ume/routing"
	"github.com/go-chi/chi/v5"
	log "github.com/sirupsen/logrus"
	"net/http"
)

func main() {
	log.Info("ume >> initializing application...")

	//if client, err := mongo.CreateClient(); err != nil {
	//	panic(err)
	//}

	r := chi.NewRouter()
	r.Mount("/", routing.NewIndexRouter())

	if err := http.ListenAndServe(":3621", r); err != nil {
		panic(err)
	}
}
