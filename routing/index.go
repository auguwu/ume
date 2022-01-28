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

package routing

import (
	"floof.gay/ume/internal"
	"floof.gay/ume/util"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"net/http"
)

type IndexResult struct {
	Hi string `json:"hi"`
}

func NewIndexRouter() chi.Router {
	router := chi.NewRouter()
	router.Use(middleware.Logger)

	router.Get("/", func(w http.ResponseWriter, r *http.Request) {
		util.WriteJson(w, 200, IndexResult{Hi: "world"})
	})

	router.Get("/version", func(w http.ResponseWriter, req *http.Request) {
		util.WriteJson(w, 200, struct {
			Version   string `json:"version"`
			CommitSHA string `json:"commit_sha"`
			BuildDate string `json:"build_date"`
		}{
			Version:   internal.Version,
			CommitSHA: internal.CommitSHA,
			BuildDate: internal.BuildDate,
		})
	})

	router.Get("/health", func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte("OK"))
	})

	return router
}
