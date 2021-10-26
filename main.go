package main

import (
	"floofy.dev/ume/mongo"
	"floofy.dev/ume/routing"
	"github.com/go-chi/chi/v5"
	"github.com/joho/godotenv"
	log "github.com/sirupsen/logrus"
	"net/http"
	"os"
)

var version string
var buildDate string
var commit string

func init() {
	log.SetFormatter(&log.TextFormatter{})
	if _, err := os.Stat("./.env"); !os.IsNotExist(err) {
		err := godotenv.Load(".env"); if err != nil {
			panic(err)
		}
	}

	log.Infof("Running v%s (commit: %s) of ume (build date: %s)", version, commit, buildDate)
}

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
