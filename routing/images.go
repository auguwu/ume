package routing

import (
	floof "floofy.dev/ume/mongo"
	"floofy.dev/ume/util"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"go.mongodb.org/mongo-driver/mongo"
	"net/http"
)

type GoAwayResult struct {
	Message string `json:"message"`
}

type BucketItemResult struct {}

func NewImagesRouter(client *mongo.Client) {
	r := chi.NewRouter()
	r.Use(middleware.Logger)

	r.Get("/", func (w http.ResponseWriter, r *http.Request) {
		bucket := floof.RetrieveBucket(client); if bucket == nil {
			util.WriteJson(w, r, 500, GoAwayResult{
				Message: "unable to retrieve GridFS bucket :(",
			})
		}
	})

	r.Post("/upload", func (w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("Authorization") == "" {
			util.WriteJson(w, r, 401, GoAwayResult{
				Message: "not allowed in this town buckeroo.",
			})
		}

		if r.Header.Get("Authorization") != "somesecurekeyiswear" {
			util.WriteJson(w, r, 403, GoAwayResult{
				Message: "you aren't allowed here buckeroo.",
			})
		}
	})
}
