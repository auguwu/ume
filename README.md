# i.noel.ws
[![Discord](https://discordapp.com/api/guilds/382725233695522816/embed.png)](https://discord.gg/yDnbEDH) [![Travis](https://travis-ci.org/auguwu/i.augu.dev.svg?branch=master)](https://travis-ci.org/github/auguwu/i.augu.dev)

> :sparkling_heart: **| ShareX uploader for personal use.**

## Installation
### Requirements
- Node.js v10+
- MongoDB v3.6+

### Process
- Fork the repository [here](https://github.com/auguwu/i.augu.dev/fork)
- Clone the forked repository (``git clone https://github.com/$USERNAME/i.augu.dev``)
- Change the directory to `i.noel.ws` and run `npm i` to install the local dependencies
  - Note: If you don't have ESLint installed, run `npm i -g eslint`!
- Change the directory to the `src` folder and run `node app.js`

### ShareX Configuration
```json
{
  "Version": "13.0.1",
  "Name": "<name>",
  "DestinationType": "ImageUploader, FileUploader",
  "RequestMethod": "POST",
  "RequestURL": "<url>",
  "Headers": {
    "Authorization": "<key u generated>"
  },
  "Body": "MultipartFormData",
  "FileFormName": "fdata",
  "URL": "<url>/uploads/$json:filename$"
}
```

### Master Key
If you want to generate a key instead of spamming your keyboard, run this in a terminal: `node -e 'console.log(require("crypto").randomBytes(32).toString("hex"))'`

### Server Configuration
```js
{
  "environment": "development", // The environment of the server
  "dbUrl": "mongodb://localhost:27017", // The database URL of the MongoDB instance
  "port": 7795, // The port to run the server in
  "key": "" // The master key to upload files
}
```

## License
This service is released under the **MIT** License, view more information [here](/LICENSE)
