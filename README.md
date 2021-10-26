# ume (梅, "plum")
> 💖 **梅 / ume is an easy, simple, and flexible image and file host.**

## Installation
You are required a **MongoDB** connection (Atlas also works!) and **Go** 1.16 or higher.

### Docker
**Docker** is required on your system to use the image.

```sh
# 1. Pull from Docker Hub
$ docker pull auguwu/ume:latest

# 2. Run the image!
$ docker run -d -p 3621:3621 --name ume --restart always \
  -v /path/to/.env:/app/ume/.env \
  auguwu/ume
```

### Git repository
If you're using Windows, you need **make** installed in your system. You can use chocolatey to do so:

> `$ choco install make`

```sh
# 1. Pull the repository
$ git clone https://github.com/auguwu/ume && cd ume

# 2. Pull the dependencies
$ go get

# 3. Run `make build` to build a binary in `./build/ume`
$ make build

# 4. Run the binary!
$ ./build/ume # append `.exe` if on Windows
```

## Configuration
**ume** can be configured using Environment Variables. Provide a `.env` file in the root directory
of where you cloned **ume** in:

```env
# Authorization key when uploading images.
AUTH=(run `node -e 'process.stdout.write(require("crypto").randomBytes(32).toString("hex") + "\n")'` to generate one!)

# The database name when connecting to MongoDB
DB=ume

# The database connection URI, must start with `mongodb` or `mongodb+srv`.
DB_URL=mongodb://localhost:27017
```

## License
**梅 ("plum")** is released under MIT License. :D
