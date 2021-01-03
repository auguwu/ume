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

const { DefaultConfig } = require('./constants');
const { promises: fs } = require('fs');
const { join } = require('path');
const leeks = require('leeks.js');
const yaml = require('js-yaml');

const log = (message) => process.stdout.write(`${leeks.colors.cyan('[config]')}   ${message}\n`);

const argv = process.argv.slice(2);
if (argv[0] !== 'create') {
  log(`${leeks.colors.red('fatal')}: invalid command.`);
  process.exit(1);
}

if (argv[0] === 'create') {
  create().then(() => process.exit(0)).catch((error) => {
    log(`${leeks.colors.red('fatal')}: unable to create`);
    console.error(error);

    process.exit(1);
  });
}

async function create() {
  log('creating default config...');

  const path = join(__dirname, '..', 'src', 'config.yml');
  const config = yaml.dump(DefaultConfig, { indent: 4 });
  await fs.writeFile(path, config);

  log('create default config file.');
}
