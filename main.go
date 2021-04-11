package main

import (
	"floofy.dev/ume/mongo"
	"floofy.dev/ume/routing"
	"github.com/go-chi/chi/v5"
	log "github.com/sirupsen/logrus"
	"net/http"
)

func main() {
	log.Info("ume >> initializing application...")

	client, err := mongo.CreateClient(); if err != nil {
		panic(err)
	}

	r := chi.NewRouter()
	r.Mount("/", routing.NewIndexRouter())
	r.Mount("/images", routing.NewImagesRouter(client))

	if err := http.ListenAndServe(":3621", r); err != nil {
		panic(err)
	}
}
