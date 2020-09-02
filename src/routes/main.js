/* eslint-disable no-invalid-this */
const { promises: fs, existsSync } = require('fs');
const { Router, Route } = require('../structures/Route');
const { join } = require('path');
const util = require('../util');

const uploads = join(process.cwd(), 'uploads');

const router = new Router('/')
  .addRoute(new Route('/', 'get', (_, res) => res.status(204).send()))
  .addRoute(new Route('/:file', 'get', async function (req, res) {
    const uuid = req.params.file;
    if (!existsSync(join(uploads, uuid))) return res.status(404).json({
      message: `File "${uuid}" doesn't exist`
    });

    return res.sendFile(join(uploads, uuid));
  }))
  .addRoute(new Route('/upload', 'post', async function (req, res) {
    const file = req.files[Object.keys(req.files)[0]];
    const ext = file.name.split('.').pop();

    if (!['png', 'jpg', 'webp', 'gif'].includes(ext)) return res.status(406).json({
      message: `You must have a valid file type (${['png', 'jpg', 'webp', 'gif'].join(' | ')})`
    });

    if (!req.headers.hasOwnProperty('authorization')) return res.status(401).json({
      message: 'No, I will not let you upload files without an API key'
    });

    if (req.headers.authorization !== this.config.key) return res.status(403).json({
      message: 'No, I will not let you upload files.'
    });

    const name = util.generate();
    const f = util.getArbitrayPath('uploads', `${name}.${ext}`);

    if (!existsSync(util.getArbitrayPath('uploads'))) await fs.mkdir(util.getArbitrayPath('uploads'));
    if (existsSync(f)) return res.status(400).json({
      statusCode: 400,
      message: `File "${f}" exists already, UUID cannot be taken.`
    });
    
    await fs.writeFile(f, file.data);
    return res.status(201).json({
      filename: `${name}.${ext}`
    });
  }));

module.exports = router;
