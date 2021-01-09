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

import { readFileSync, existsSync } from 'fs';
import { join } from 'path';
import yaml from 'js-yaml';

export interface Configuration {
  environment: 'development' | 'production';
  ratelimits?: RatelimitConfiguration;
  sentryDSN?: string;
  uploads: UploadsConfiguration;
  port: number;
  ssl?: SSLConfiguration;
  gc?: GarbageCollectorConfiguration;
}

export interface SSLConfiguration {
  cert: string;
  key: string;
  ca?: string;
}

interface RatelimitConfiguration {
  requests: number;
  time: string | number;
}

interface UploadsConfiguration {
  filesystem: FilesystemUploadConfiguration;
  gcs: GoogleCloudUploadConfiguration;
}

interface FilesystemUploadConfiguration {
  uploads: string;
}

interface GoogleCloudUploadConfiguration {
  a: 'b';
}

interface GarbageCollectorConfiguration {
  interval: string | number;
  enabled: boolean;
}

export default class Config {
  private cache!: Configuration;

  load() {
    const configPath = join(__dirname, '..', 'config.yml');
    if (!existsSync(configPath))
      throw new Error(`Missing configuration file in \`${configPath}\`.`);

    const contents = readFileSync(configPath, 'utf8');
    this.cache = yaml.load(contents);
  }

  get<T>(key: string, defaultValue: T): T;
  get<T>(key: string): T | null;
  get<T>(key: string, defaultValue?: T) {
    const nodes = key.split('.');
    let prop: any = this.cache;

    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      try {
        prop = prop[node];
      } catch {
        prop = '<not found>';
        break;
      }
    }

    if (prop === '<not found>') throw new TypeError(`Couldn't find anything in nodes '${nodes.join('.')}'`);

    if (defaultValue) {
      return (prop === null || prop === void 0) ? defaultValue : prop;
    } else {
      return (prop === null || prop === void 0) ? null : prop;
    }
  }
}
