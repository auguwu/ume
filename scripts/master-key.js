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

const { promises: fs } = require('fs');
const { randomBytes } = require('crypto');
const { join } = require('path');
const leeks = require('leeks.js');
const yaml = require('js-yaml');

const log = (message) => process.stdout.write(`${leeks.colors.cyan('[master:key]')}   ${message}\n`);

async function main() {
  log('generating master key...');

  let config;
  try {
    config = await fs.readFile(join(__dirname, '..', 'src', 'config.yml'), 'utf8');
  } catch(ex) {
    log(`${leeks.colors.red('fatal')}: Did you generate a configuration file? Run \`npm run config:create\` to create one.`);
    process.exit(1);
  }

  const c = yaml.safeLoad(config);
  log('successfully loaded configuration');

  if (!c.gc.enabled) {
    log('Garbage Collector isn\'t enabled, not creating one.');
    process.exit(1);
  }

  const key = randomBytes(16).toString('hex');
  c.gc.master_key = key; // eslint-disable-line camelcase

  yaml.safeDump(c, { indent: 4 });
  log(`master key is set to \`${key}\`, use this with the /users/create endpoint.`);
}

main();
