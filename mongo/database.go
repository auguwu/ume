package mongo

import (
	"context"
	log "github.com/sirupsen/logrus"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/gridfs"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.mongodb.org/mongo-driver/mongo/readpref"
	"time"
)

func CreateClient() (*mongo.Client, error) {
	log.Info("ume >> connecting -> mongodb")

	uri := "mongodb://localhost:27017"
	ctx, cancel := context.WithTimeout(context.TODO(), 2 * time.Second)
	defer cancel()

	client, err := mongo.Connect(ctx, options.Client().ApplyURI(uri).SetAppName("Ume")); if err != nil {
		log.Fatal(err)
	}

	if err := client.Ping(ctx, readpref.Primary()); err != nil {
		log.Fatal("ume >> connection isn't established!")
		return nil, err
	}

	log.Info("ume >> connected -> mongodb")
	return client, nil
}

func RetrieveBucket(client *mongo.Client) *gridfs.Bucket {
	// TODO: add custom database name?
	if bucket, err := gridfs.NewBucket(client.Database("ume"), options.GridFSBucket().SetName("images")); err != nil {
		return nil
	} else {
		return bucket
	}
}
