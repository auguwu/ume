package mongo

import (
	"context"
	log "github.com/sirupsen/logrus"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/gridfs"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.mongodb.org/mongo-driver/mongo/readpref"
	"os"
	"time"
)

func CreateClient() (*mongo.Client, error) {
	log.Info("ume >> connecting -> mongodb")

	ctx, cancel := context.WithTimeout(context.TODO(), 2*time.Second)
	defer cancel()

	client, err := mongo.Connect(ctx, options.Client().ApplyURI(os.Getenv("DB_URL")).SetAppName("Ume"))
	if err != nil {
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
	if bucket, err := gridfs.NewBucket(client.Database(os.Getenv("DB")), options.GridFSBucket().SetName("images")); err != nil {
		return nil
	} else {
		return bucket
	}
}
