# cute.floofy.dev
> :sparkling_heart: **Minimal, yet powerful image hosting server made with TypeScript**

## Installation
### Requirements
- PostgreSQL (optional for Registration)
- Node.js v10+
- Docker (optional)
- Sentry (optional)

### Process (locally)
- [Fork](https://github.com/auguwu/cute.floofy.dev/fork) the repository to your user account or organization
- Clone the repository (``git clone https://github.com/<username>/cute.floofy.dev``), omit `<username>` with the user you cloned the repository with.
- Change the directory to the server (``cd cute.floofy.dev``) and run `npm install` to install all the dependencies
- Run `npm run build` to compile the source code to runnable JavaScript code
- Run `npm run config:create` to create a configuration file, it'll be placed in `src/config.yml` for moving easily.
- Run `npm start` to run the server, it'll attempt to move `src/config.yml` to `build/` and start the server.

### Process (Docker, locally)
> This process is the most recommended for Docker users and production use.

- [Fork](https://github.com/auguwu/cute.floofy.dev/fork) the repository to your user account or organization
- Clone the repository (``git clone https://github.com/<username>/cute.floofy.dev``), omit `<username>` with the user you cloned the repository with.
- Change the directory to the server (``cd cute.floofy.dev``)
- Run `npm i` to install local dependencies
- Run `npm run config:create` to create the configuration file
- Run `docker build . -t sharex:latest` to build the image
- Run `docker run -d -p <port to use>:2546 sharex:latest` to run the image
  - If you don't use **Google Cloud Storage**, you must add a volume to that command, so add the following command when running the container:
  - `-v <path to your images>:/uploads`

## Configuration
This is an example of a detailed configuration file and it shows documentation about what each setting does

```yml
# The environment, depends on logging mainly
environment: "development"

# (Optional) The DSN URI to report errors when anything happens
sentryDSN: ""

# The port to connect to
port: 2546

# Uploads configuration
uploads:
  # GCS Configuration
  gcs:
    # TODO: this

  # Local FS settings
  filesystem:
    # The directory to upload all files in
    directory: ./uploads

# Garbage Collector Settings
gc:
  # If it should be enabled
  # This is always false when you are using GCS
  enabled: false

  # The interval to delete images
  # It can take a string (example "1h") or a time in milliseconds (example "60000")
  interval: 1h

# Ratelimiting settings
ratelimits:
  # The amount of time per request (i.e 1,500 requests per 5 minutes)
  # It can take a string (example "1h") or a time in milliseconds (example "60000")
  time: 3d

  # Amount of requests until we abort the request
  requests: 1500
```

## License
**cute.floofy.dev** is released under the [**MIT**](/LICENSE) License. :)
