const { Collection } = require('@augu/immutable');
const fileUpload = require('express-fileupload');
const Database = require('./Database');
const express = require('express');
const Logger = require('./Logger');
const routes = require('../routes');
const utils = require('../util');
const GC = require('../util/GarbageCollector');

module.exports = class Server {
  /**
   * Creates a new `Server` instance
   * @param {Config} config The config
   */
  constructor(config) {
    this.requests = 0;
    this.database = new Database(config.dbUrl);
    this.routers = new Collection();
    this.logger = new Logger('Server');
    this.config = config;
    this.app = express();
    this.gc = new GC(this);
  }

  isDev() {
    return this.config.environment === 'development';
  }

  addMiddleware() {
    // Disable this for no XSS attacks
    this.app.disable('x-powered-by');
    this.app.use(fileUpload({
      preserveExtension: true,
      safeFileNames: true
    }));
  }

  addRoutes() {
    for (const [, router] of Object.entries(routes)) {
      this.routers.set(router.route, router);
      for (const route of router.routes.values()) {
        this.logger.info(`Registered "${route.route}" to the main app instance`);
        this.app[route.method](route.route, async (req, res) => {
          try {
            this.requests++;
            await route.callee.apply(this, [req, res]);
          }
          catch(ex) {
            this.logger.error(`Unable to fulfill request to "${route.route}"`, ex);
            return res.status(500).json({
              statusCode: 500,
              message: 'Unable to fulfill request',
              error: ex.message
            });
          }
        });
      }
    }
  }

  async launch() {
    this.logger.info('Launching ShareX server...');
    
    this.addMiddleware();
    this.addRoutes();
    this.database.connect();
    this.gc.start();

    await utils.sleep(2000);
    this._server = this.app.listen(this.config.port, () =>
      this.logger.info(`Now listening on port ${this.config.port}${this.isLocalhost() ? ', running locally!' : ' (https://i.augu.dev)'}`)
    );
  }

  /**
   * Disposes any connections
   */
  dispose() {
    this.logger.warn('Disposing all instances...');

    this.gc.dispose();
    this._server.close();
    this.routers.clear();
    this.database.dispose();

    this.logger.warn('Disposed the ShareX server.');
  }
};

/**
 * @typedef {object} Config
 * @prop {"development" | "production"} environment The environment state of the server
 * @prop {string} dbUrl The database URL
 * @prop {number} port The port to use to connect
 * @prop {string} key The master key to add images
 */