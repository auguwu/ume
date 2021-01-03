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

import type { NextFunction, Request, Response } from 'express';
import { Server, Logger } from '..';
import { readdir } from '../../util';
import { join } from 'path';

type ExpressMiddleware = (
  req: Request,
  res: Response,
  next: NextFunction
) => void;

export default class EndpointHandler {
  private directory: string;
  private logger: Logger;
  private server: Server;

  constructor(server: Server) {
    this.directory = join(__dirname, '..', '..', 'middleware');
    this.server = server;
    this.logger = new Logger('Middleware');
  }

  async load() {
    this.logger.info('Loading all middleware!');

    const files = await readdir(this.directory);
    if (!files.length) {
      this.logger.warn('Missing middleware! Do you have a corrupt installation?');
      return;
    }

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const ctor = await import(file);

      if (!ctor.default) {
        this.logger.warn(`Middleware at "${file}" is missing a default export`);
        continue;
      }

      const middleware: ExpressMiddleware = ctor.default;
      this.server.app.use(middleware);
    }

    this.logger.info('Loaded all middleware!');
  }
}
