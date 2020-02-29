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
  start() {
    this.logger.info('Garbage collecting has started!');
    this.interval = setInterval(async () => {
      const cwd = utils.getArbitrayPath('uploads');
      const files = await fs.readdir(cwd);
      this.logger.info(`Found ${files.length} files to delete from cache`);
      for (const file of files) {
        this.logger.info(`Now deleting file ${file} from the database!`);
        await fs.unlink(join(cwd, file))
          .catch(error => this.logger.error(`Unable to delete file ${file}:`, error));

        const uuid = file.split('.').shift();
        await this.server.database.delImage(uuid);

        this.logger.info(`Deleted file ${file} from cache.`);
      }
    }, 604800000);
  }

  /**
   * Disposes the GC instance
   */
  dispose() {
    clearInterval(this.interval);
    this.logger.warn('Disposed the garbage collector');
  }
};