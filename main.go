// 💖 ume: Easy, self-hostable, and flexible image and file host, made in Go using MongoDB GridFS.
// Copyright (c) 2020-2022 Noel <cutie@floofy.dev>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

package main

import (
	"context"
	"floof.gay/ume/internal"
	"floof.gay/ume/mongo"
	"floof.gay/ume/routing"
	"fmt"
	"github.com/go-chi/chi/v5"
	"github.com/joho/godotenv"
	log "github.com/sirupsen/logrus"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"syscall"
	"time"
)

func init() {
	log.SetFormatter(&log.TextFormatter{})
	if _, err := os.Stat("./.env"); !os.IsNotExist(err) {
		err := godotenv.Load(".env")
		if err != nil {
			panic(err)
		}
	}

	log.Infof("Running v%s (commit: %s) of ume (build date: %s)", internal.Version, internal.CommitSHA, internal.BuildDate)
}

func main() {
	log.Info("ume >> initializing application...")

	client, err := mongo.CreateClient()
	if err != nil {
		panic(err)
	}

	r := chi.NewRouter()
	r.Mount("/", routing.NewIndexRouter())
	r.Mount("/images", routing.NewImagesRouter(client))

	port := 3621
	if _, ok := os.LookupEnv("PORT"); ok {
		raw, err := strconv.Atoi(os.Getenv("PORT"))
		if err != nil {
			log.Fatalf("Unable to parse PORT environment variable: %v", err)
		}

		port = raw
	}

	h := ""
	if host, ok := os.LookupEnv("HOST"); ok {
		h = host
	}

	addr := fmt.Sprintf("%s:%d", h, port)
	server := &http.Server{
		Addr:         addr,
		Handler:      r,
		WriteTimeout: 10 * time.Second,
	}

	sigint := make(chan os.Signal, 1)
	signal.Notify(sigint, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		log.Infof("Now listening under '%s'", addr)
		err = server.ListenAndServe()
		if err != nil && err != http.ErrServerClosed {
			log.Errorf("Unable to run server: %s", err)
		}
	}()

	<-sigint

	log.Warn("Told to close off server...")
	shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)

	// Wait for connections to die off
	go func() {
		<-shutdownCtx.Done()
		if shutdownCtx.Err() == context.DeadlineExceeded {
			log.Warn("Received deadline to close off incoming requests...")
		}
	}()

	defer func() {
		err = client.Disconnect(context.TODO())
		if err != nil {
			log.Errorf("Unable to shutdown mongo client: %v", err)
		}

		log.Info("Closed off MongoDB connection!")
		cancel()
	}()

	if err = server.Shutdown(shutdownCtx); err != nil {
		panic(err)
	}

	log.Info("Closed off server!")
}
