const { promises: fs, existsSync } = require('fs');
const { Router, Route } = require('../structures/Route');
const util = require('../util');

const router = new Router('/')
  .addRoute(new Route('/', 'get', async function (_, res) {
    const files = await this.database.images.find({}).toArray();
    return res.status(200).json({
      message: 'Um, I have no idea why you are here, b-but hello?',
      requests: this.requests,
      files: files.length
    });
  }))
  .addRoute(new Route('/uploads/:file', 'get', async function (req, res) {
    const uuid = req.params.file.split('.').shift();
    const image = await this.database.getImage(uuid);

    if (!image || image === null) return res.status(404).json({
      statusCode: 404,
      message: `Unable to find image "${req.params.file}"`
    });

    return res.sendFile(image.path);
  }))
  .addRoute(new Route('/file/:file', 'get', async function (req, res) {
    const uuid = req.params.file.split('.').shift();
    const image = await this.database.getImage(uuid);

    if (!image || image === null) return res.status(404).json({
      statusCode: 404,
      message: `Unable to find image with UUID "${req.params.file}"`
    });

    return res.status(200).json({
      statusCode: 200,
      data: {
        createdAt: image.createdAt,
        size: util.formatSize(image.size),
        file: this.isDev() ? `http://localhost:${this.config.port}/uploads/${image.uuid}.${image.ext}` : `https://i.augu.dev/uploads/${image.uuid}.${image.ext}`
      }
    });
  }))
  .addRoute(new Route('/upload', 'post', async function (req, res) {
    const file = req.files[Object.keys(req.files)[0]];
    const ext = file.name.split('.').pop();

    if (!['png', 'jpg', 'webp', 'gif'].includes(ext)) return res.status(406).json({
      statusCode: 406,
      message: `You must have a valid file type (${['png', 'jpg', 'webp', 'gif'].join(' | ')})`
    });

    if (!req.headers.authorization) return res.status(401).json({
      statusCode: 401,
      message: 'No, I will not let you upload files without an API key'
    });

    if (req.headers.authorization !== this.config.key) return res.status(403).json({
      statusCode: 403,
      message: 'No, I will not let you upload files.'
    });

    const name = util.generate();
    const f = util.getArbitrayPath('uploads', `${name}.${ext}`);

    if (!existsSync(util.getArbitrayPath('uploads'))) await fs.mkdir(util.getArbitrayPath('uploads'));
    await fs.writeFile(f, file.data);

    this.database.addImage({
      createdAt: util.dateformat(Date.now()).toString('mm/dd/yyyy hh:MM:ss TT'),
      size: file.data.length,
      uuid: name,
      path: f,
      ext
    });

    return res.status(201).json({
      statusCode: 201,
      filename: `${name}.${ext}`
    });
  }));

module.exports = router;