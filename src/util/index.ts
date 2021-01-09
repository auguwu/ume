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

import { promises as fs } from 'fs';
import { randomBytes } from 'crypto';
import { join } from 'path';
import ms from 'ms';

/**
 * Returns a boolean-represented value if the user is running
 * Node.js v10 or higher based on their process' version.
 *
 * @param version The version to use, default is `process.version`
 */
export function isNode10(version: string = process.version) {
  const major = Number(version?.split('.')[0].replace('v', ''));
  return major === 10 || major > 10;
}

/**
 * Generates a random key for the [FilesystemProvider] provider.
 */
export const generate = () =>
  randomBytes(3).toString('hex');

/**
 * Pauses the event loop to run this macrotask.
 *
 * All promises are a macrotask while a setTimeout/setInterval is a microtask;
 * This function pauses the callback loop but lets the synchronous code run
 * but it'll pause until the duration has passed.
 *
 * @param duration The amount of milliseconds to pause this loop
 * @returns Returns an unknown promise. It makes sure
 * that runtime will know that this is an empty fulfilled promise.
 */
export const sleep = (duration: number) => new Promise(resolve => setTimeout(resolve, duration));

/**
 * Formats any byte-integers to a humanizable string
 * @param bytes The number of bytes the file is
 */
export const formatSize = (bytes: number) => {
  const kilo = bytes / 1024;
  const mega = kilo / 1024;
  const giga = mega / 1024;

  if (kilo < 1024) return `${kilo.toFixed(1)}KB`;
  if (kilo > 1024 && mega < 1024) return `${mega.toFixed(1)}MB`;
  else return `${giga.toFixed(1)}GB`;
};

/**
 * Asynchronouslly read a directory recursively if any directories
 * are hit. [fs.readdir] does the job but doesn't recursively
 * add them in the array once fetched, it's just the directory name,
 * not the contents of that directory.
 *
 * @param path The path to get all files from
 */
export async function readdir(path: string) {
  let results: string[] = [];
  const files = await fs.readdir(path);

  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    const stats = await fs.lstat(join(path, file));

    if (stats.isDirectory()) {
      const r = await readdir(join(path, file));
      results = results.concat(r);
    } else {
      results.push(join(path, file));
    }
  }

  return results;
}

export function convertTime(timestamp: string | number) {
  if (!['string', 'number'].includes(typeof timestamp)) throw new TypeError('expected `string` or `number`');

  return typeof timestamp === 'string'
    ? ms(timestamp)
    : timestamp;
}
