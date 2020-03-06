const { promises: fs } = require('fs');
const { join } = require('path');
const Logger = require('../structures/Logger');
const utils = require('.');

module.exports = class GarbageCollector {
  /**
   * Creates a new instance of the Garbage Collector class
   * @param {import('../structures/Server')} server The server instance
   */
  constructor(server) {
    this.interval = null;
    this.logger = new Logger('GC');
    this.server = server;
  }

  /**
   * Starts the garbage collecting
   */
  async start() {
    this.logger.info('Garbage collecting has started!');
    const files = await fs.readdir(utils.getArbitrayPath('uploads'));
    if (files.length && files.length > 25) {
      this.logger.warn('Detected more then 25 files were detected, now removing...');
      await this.onTrashed(utils.getArbitrayPath('uploads'));
    }

    this.interval = setInterval(async () => {
      this.logger.info('Garbage collecting is in progress...');
      await this.onTrashed(utils.getArbitrayPath('uploads'));
    }, 604800000);
  }

  /**
   * Disposes the GC instance
   */
  dispose() {
    clearInterval(this.interval);
    this.logger.warn('Disposed the garbage collector');
  }

  /**
   * Does the collecting and removes the file
   * @param {string} cwd The directory to remove the files
   */
  async onTrashed(cwd) {
    const files = await fs.readdir(cwd);
    this.logger.info(`Found ${files.length} files to delete from cache!`);

    for (const file of files) {
      this.logger.warn(`Now deleting file ${file} from the database...`);
      await fs.unlink(join(cwd, file))
        .catch(error => this.logger.error(`Unable to delete file ${file}:`, error));

      const uuid = file.split('.').shift();
      const image = await this.server.database.getImage(uuid);
      await this.server.database.delImage(uuid);

      this.logger.info(`Deleted image ${image.uuid} (size=${image.size},extension=${image.ext})`);
    }
  }
};