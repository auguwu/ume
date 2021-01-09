/**
 * Copyright (c) 2020-2021 August
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

import GoogleCloudProvider from './providers/GCSProvider';
import FilesystemProvider from './providers/FilesystemProvider';
import RatelimitHandler from './RatelimitHandler';
import EndpointManager from './managers/EndpointManager';
import type Provider from './Provider';
import fileUpload from 'express-fileupload';
import express from 'express';
import Logger from './Logger';
import Config from './Config';
import http from 'http';
import os from 'os';

import * as middleware from '../middleware';

export type Middleware = (this: Server) =>
  (req: express.Request, res: express.Response, next: express.NextFunction) => void;

type ProviderObject = {
  [x in 'gcs' | 'filesystem']?: any;
}

export default class Server {
  public ratelimits: RatelimitHandler;
  public endpoints: EndpointManager;
  public provider!: Provider;
  private _server!: http.Server;
  public logger: Logger;
  public config: Config;
  public app: express.Application;

  constructor() {
    this.config = new Config();
    this.config.load();

    this.ratelimits = new RatelimitHandler(this);
    this.endpoints = new EndpointManager(this);
    this.logger = new Logger('Server');
    this.app = express();
  }

  private findProvider(): Provider | null {
    const provider = this.config.get<ProviderObject>('uploads');
    if (provider === null) throw new TypeError('Missing provider in `uploads` configuration');

    const keys = Object.keys(provider).filter(r => r !== 'extensions');
    if (keys.length > 1) throw new TypeError('Must only have 1 provider registered');

    switch (keys[0]) {
      case 'filesystem': return new FilesystemProvider().init(this);
      //case 'gcs': return new GoogleCloudProvider().init(this);
      default: return null;
    }
  }

  async start() {
    const env = this.config.get<'development' | 'production'>('environment', 'development');
    process.env.NODE_ENV = env;

    if (env === 'development')
      this.logger.warn('You are running a development build of this server, if any errors occur -- report using the URL below', {
        reportUrl: 'https://github.com/auguwu/cute.floofy.dev/issues'
      });

    this.logger.info('Booting up server...');
    this.config.load();

    const provider = this.findProvider();
    if (provider === null) throw new TypeError('Provider that was provided isn\'t supported');

    const mods = Object.values(middleware);
    for (const mod of mods) {
      this.app.use(mod.call(this));
    }

    this.app.use(this.ratelimits.middleware);
    this.app.use(fileUpload());

    await this.endpoints.load();
    this.provider = provider;
    this._server = http.createServer(this.app);

    await this.provider.start();
    const port = this.config.get<number>('port', 3621);
    const prefix = 'http';

    this
      ._server
      .on('error', (error) => this.logger.error('Unexpected error has occured', error))
      .once('listening', () => {
        const address = this._server.address();
        const networks: string[] = [];

        if (typeof address === 'string') {
          networks.push(`• UNIX Sock "${address}"`);
        } else if (address !== null) {
          if (address.address === '::')
            networks.push(`• Local: ${prefix}://localhost:${port}`);
          else
            networks.push(`• Network: ${prefix}://${address.address}:${port}`);
        }

        const external = this._getExternalNetwork();
        if (external !== null) networks.push(`• Network: ${prefix}://${external}:${port}`);

        this.logger.info('Listening under available connections', networks);
      });

    this._server.listen(port);
  }

  close() {
    this.ratelimits.dispose();
    this._server.close();
  }

  private _getExternalNetwork() {
    const interfaces = os.networkInterfaces();

    for (const name of Object.keys(interfaces)) {
      for (const i of interfaces[name]!) {
        const { address, family, internal } = i;
        if (family === 'IPv4' && !internal) return address;
      }
    }

    return null;
  }
}
