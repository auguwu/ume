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

import { existsSync, promises as fs } from 'fs';
import * as util from '../../util';
import Provider from '../Provider';
import { join } from 'path';

export default class FilesystemProvider extends Provider {
  constructor() {
    super('filesystem');
  }

  private get path() {
    return this
      .server
      .config
      .get<string>('uploads.filesystem.directory', '$(root)/uploads')
      .replace(/[$]\(([\w\.]+)\)/g, (_, key) => {
        if (key === 'root') return process.cwd();
        return key;
      });
  }

  async addFile(data: any) {
    const id = util.generate();

    await fs.writeFile(join(this.path, `${id}.png`), data);
    return join(this.path, `${id}.png`);
  }

  files() {
    return util.readdir(this.path).then((files) => files.length);
  }

  async start() {
    if (!existsSync(this.path)) await fs.mkdir(this.path);
  }
}
