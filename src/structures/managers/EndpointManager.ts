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

import { Endpoint, Server, Logger } from '..';
import { getRouteDefinitions } from '../decorators/Route';
import { Collection } from '@augu/collections';
import { readdir } from '../../util';
import { join } from 'path';

export default class EndpointHandler extends Collection<string, Endpoint> {
  private directory: string;
  private logger: Logger;
  private server: Server;

  constructor(server: Server) {
    super();

    this.directory = join(__dirname, '..', '..', 'endpoints');
    this.server = server;
    this.logger = new Logger('Endpoints');
  }

  async load() {
    this.logger.info('Initializing all endpoints...');

    const files = await readdir(this.directory);
    if (!files.length) {
      this.logger.warn('Missing endpoints! Do you have a corrupt installation?');
      return;
    }

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const ctor = await import(file);

      if (!ctor.default) {
        this.logger.warn(`Endpoint at "${file}" is missing a default export`);
        continue;
      }

      const endpoint: Endpoint = new ctor.default().init(this.server);
      const routes = getRouteDefinitions(endpoint);
      endpoint.append(routes);

      for (let i = 0; i < routes.length; i++) {
        const route = routes[i];
        const prefix = Endpoint._mergePrefix(endpoint, route.endpoint);

        this.logger.info(`Found route "${route.method} ${prefix}" in endpoint ${endpoint.prefix}`);
        this.server.app[route.method](prefix, (req, res) => {
          try {
            route.run.call(endpoint, req, res);
          } catch(ex) {
            this.logger.error(`Unexpected error while running "${route.method.toUpperCase()} ${prefix}"`, ex);
            return res.status(500).json({
              message: 'Unexpected error has occured',
              error: `[${ex.name}] ${ex.message.slice(ex.name.length + 1)}`,
              code: 'UNEXPECTED_SERVER_ERROR'
            });
          }
        });
      }

      this.set(endpoint.prefix, endpoint);
      this.logger.info(`Added endpoint "${endpoint.prefix}" successfully with ${routes.length} routes.`);
    }
  }
}
