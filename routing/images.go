package routing

import (
	"context"
	floof "floofy.dev/ume/mongo"
	"floofy.dev/ume/util"
	"fmt"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/sirupsen/logrus"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
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
		w.WriteHeader(200)
	})

	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		bucket := floof.RetrieveBucket(client)
		if bucket == nil {
			util.WriteJson(w, 500, Response{
				Message: "Failed to get bucket!",
			})
			return
		}

		cursor, err := bucket.GetFilesCollection().Find(context.TODO(), bson.D{}, options.Find().SetProjection(map[string]int{
			"filename": 1,
		}))
		if err != nil {
			logrus.Errorf("Failed to lookup images: %s", err.Error())
			return
		}

		var arr []BucketItemResult
		if err := cursor.All(context.TODO(), &arr); err != nil {
			logrus.Errorf("Failed to decode images: %s", err.Error())
			util.WriteJson(w, 200, []BucketItemResult{})
			return
		}

		if arr == nil {
			util.WriteJson(w, 200, []BucketItemResult{})
			return
		}
		util.WriteJson(w, 200, arr)
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
