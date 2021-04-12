package routing

import (
	"floofy.dev/ume/util"
	chi "github.com/go-chi/chi/v5"
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

	return router
}
