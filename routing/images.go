package routing

import (
	"context"
	floof "floofy.dev/ume/mongo"
	"floofy.dev/ume/util"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"net/http"
)

type GoAwayResult struct {
	Message string `json:"message"`
}

type BucketItemResult struct {}

func NewImagesRouter(client *mongo.Client) chi.Router {
	r := chi.NewRouter()
	r.Use(middleware.Logger)

	r.Get("/", func (w http.ResponseWriter, r *http.Request) {
		bucket := floof.RetrieveBucket(client); if bucket == nil {
			return
		}

		cursor, err := bucket.GetFilesCollection().Find(context.TODO(), bson.D{}); if err != nil {
			print(err.Error())
			return
		}

		var arr []bson.M
		if err := cursor.All(context.TODO(), &arr); err != nil {
			println(err.Error())
			util.WriteJson(w, r, 200, []BucketItemResult{})
		}

		util.WriteJson(w, r, 200, arr)
	})

	r.Post("/upload", func (w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("Authorization") == "" {
			util.WriteJson(w, r, 401, GoAwayResult{
				Message: "not allowed in this town buckeroo.",
			})
		}

		auth := util.Auth()
		if r.Header.Get("Authorization") != auth {
			util.WriteJson(w, r, 403, GoAwayResult{
				Message: "you aren't allowed here buckeroo.",
			})
		}

		bucket := floof.RetrieveBucket(client); if bucket == nil {
			return
		}

		//data, header, _ := r.FormFile("fdata")
		//file := util.RandomString()
		//if _, err := bucket.UploadFromStream(file, ); err != nil {
		//	return
		//}
	})

	return r
}
