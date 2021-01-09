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

import type { Request, Response } from 'express';
import { Endpoint, Route } from '../structures';

const pkg = require('../../package.json');

export default class MainEndpoint extends Endpoint {
  constructor() {
    super('/');
  }

  @Route('get', '/')
  async main(_: Request, res: Response) {
    return res.status(200).json({
      message: 'henlo, iz weh!',
      version: `v${pkg.version}`
    });
  }

  @Route('get', '/stats')
  async stats(_: Request, res: Response) {
    const files = await Promise.resolve(this.server.provider.files());
    return res.status(200).json({
      files
    });
  }

  @Route('post', '/upload')
  async upload(req: Request, res: Response) {
    // i need to add auth here lol
  }
}
