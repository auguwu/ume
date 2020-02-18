const { promises: fs, existsSync } = require('fs');
const { Router, Route } = require('../structures/Route');
const util = require('../util');

const router = new Router('/')
  .addRoute(new Route('/', 'get', async function (_, res) {
    const files = await this.database.images.find({}).toArray();
    return res.status(200).json({
      message: 'Hello User! I don\'t think you should be here...',
      requests: this.requests.toLocaleString(),
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
  .addRoute(new Route('/upload', 'post', async function (req, res) {
    const file = req.files[Object.keys(req.files)[0]];
    const ext = file.name.split('.').pop();

    if (req.headers.authorization !== this.config.key) return res.status(403).json({
      statusCode: 403,
      message: 'No, I will not let you upload files.'
    });

    const name = util.generate();
    if (!existsSync(util.getArbitrayPath('uploads'))) await fs.mkdir(util.getArbitrayPath('uploads'));
    await fs.writeFile(util.getArbitrayPath('uploads', `${name}.${ext}`), file.data);

    const path = util.getArbitrayPath('uploads', `${name}.${ext}`);
    this.database.addImage({
      createdAt: util.dateformat(Date.now()).toString('mm/dd/yyyy hh:MM:ssTT'),
      size: file.data.length,
      uuid: name,
      path
    });

    return res.status(201).json({
      statusCode: 201,
      filename: `${name}.${ext}`
    });
  }));

module.exports = router;