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

import { Logger, Server } from './structures';
import * as util from './util';

const logger = new Logger('Master');

async function main() {
  if (!util.isNode10()) return Promise.reject(new Error(`Machine is not running Node.js v10 or higher (${process.version})`));

  logger.info('Loading server...');

  const server = new Server();
  try {
    await server.start();
  } catch(ex) {
    logger.error('Unexpected error has occured while running', ex);
    process.exit(1);
  }

  process.on('unhandledRejection', error => logger.error(error));
  process.on('uncaughtException', error => logger.error(error));
  process.on('SIGINT', () => {
    logger.warn('Closing server');
    server.close();

    process.exit(1);
  });
}

main()
  .catch(error => {
    logger.error(error);
    process.exit(1);
  });
