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

import type { RouteDefinition } from './decorators/Route';
import type Server from './Server';
import { Collection } from '@augu/collections';

export default class Endpoint {
  public server!: Server;
  public prefix: string;
  public routes: Collection<string, RouteDefinition>;

  static _mergePrefix(endpoint: Endpoint, path: string) {
    if (endpoint.prefix === path) return endpoint.prefix;
    return `${endpoint.prefix === '/' ? '' : endpoint.prefix}${path}`;
  }

  constructor(prefix: string) {
    this.prefix = prefix;
    this.routes = new Collection();
  }

  init(server: Server) {
    this.server = server;
    return this;
  }

  append(routes: RouteDefinition[]) {
    for (let i = 0; i < routes.length; i++) {
      const route = routes[i];
      const path = Endpoint._mergePrefix(this, route.endpoint);

      this.routes.emplace(path, route);
    }
  }
}
