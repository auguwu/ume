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
	floof "floof.gay/ume/mongo"
	"floof.gay/ume/util"
	"fmt"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/sirupsen/logrus"
	"go.mongodb.org/mongo-driver/mongo"
	"io"
	"net/http"
	"strings"
)

var types = map[string]string{
	"png":  "image/png",
	"jpg":  "image/jpg",
	"webp": "image/webp",
	"gif":  "image/gif",
	"mp4":  "video/mp4",
}

type Response struct {
	Message string `json:"message"`
}

type BucketItemResult struct {
	Filename string `json:"filename"`
}

func IsValidFileType(t string) bool {
	_, ok := types[t]
	return ok
}

func NewImagesRouter(client *mongo.Client) chi.Router {
	r := chi.NewRouter()
	r.Use(middleware.Logger)

	r.Get("/{id}", func(w http.ResponseWriter, r *http.Request) {
		name := chi.URLParam(r, "id")
		bucket := floof.RetrieveBucket(client)
		if bucket == nil {
			util.WriteJson(w, 500, Response{
				Message: "Failed to get bucket!",
			})
			return
		}

		stream, err := bucket.OpenDownloadStreamByName(name)
		if err != nil {
			util.WriteJson(w, 404, Response{
				Message: "Not found!",
			})
			return
		}

		s := strings.Split(name, ".")
		if len(s) < 1 || !IsValidFileType(s[len(s)-1]) {
			util.WriteJson(w, 400, Response{
				Message: fmt.Sprintf("Unknown file type %s", s[len(s)-1]),
			})
			return
		}

		_, err = io.Copy(w, stream)
		if err != nil {
			util.WriteJson(w, 500, Response{
				Message: "Failed to copy buffer!",
			})
			return
		}

		contentType, _ := types[s[len(s)-1]]

		w.Header().Set("Content-Type", contentType)
		w.Header().Set("Cache-Control", "public, max-age=777600, must-revalidate")
		w.WriteHeader(200)
	})

	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		util.WriteJson(w, 200, struct {
			Message string `json:"message"`
		}{
			Message: "hello world",
		})
	})

	r.Post("/upload", func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("Authorization") == "" || r.Header.Get("Authorization") != util.Auth() {
			util.WriteJson(w, 403, Response{
				Message: "Unauthorized",
			})
			return
		}

		bucket := floof.RetrieveBucket(client)
		if bucket == nil {
			util.WriteJson(w, 500, Response{
				Message: "Failed to get bucket!",
			})
			return
		}

		data, header, _ := r.FormFile("fdata")
		if header.Size >= 1073741824 {
			util.WriteJson(w, 400, Response{
				Message: "File size exceeds 1GB",
			})
			return
		}

		s := strings.Split(header.Filename, ".")
		if len(s) < 1 || !IsValidFileType(s[len(s)-1]) {
			util.WriteJson(w, 400, Response{
				Message: fmt.Sprintf("Unknown file type %s", s[len(s)-1]),
			})
			return
		}

		file := util.RandomString()
		if _, err := bucket.UploadFromStream(fmt.Sprintf("%s.%s", file, s[len(s)-1]), data); err != nil {
			logrus.Errorf("Failed to upload file %s (%s): %s", file, header.Filename, err.Error())
			util.WriteJson(w, 500, Response{
				Message: "Error uploading file!",
			})
			return
		}

		util.WriteJson(w, 201, BucketItemResult{
			Filename: fmt.Sprintf("%s.%s", file, s[len(s)-1]),
		})
	})

	return r
}
