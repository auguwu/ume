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

import type { Response, Request } from 'express';

type MaybePromise<T> = T | Promise<T>;
const SYMBOL = Symbol('$routes');

export interface RouteDefinition {
  run(req: Request, res: Response): MaybePromise<void>;
  endpoint: string;
  method: 'get' | 'post';
}

export function getRouteDefinitions(target: any) {
  if (target.constructor === null) return [];

  const definitions = target.constructor[SYMBOL] as RouteDefinition[];
  if (!Array.isArray(definitions)) return [];

  return definitions;
}

export default function Route(method: 'get' | 'post', endpoint: string): MethodDecorator {
  return (target, prop, descriptor) => {
    const property = String(prop);
    if ((<any> target).prototype !== undefined) throw new TypeError(`Method "${target.constructor.name}#${property}" is static`);
    if (!target.constructor[SYMBOL]) target.constructor[SYMBOL] = [];

    (target.constructor[SYMBOL] as RouteDefinition[]).push({
      run: descriptor.value as any,
      endpoint,
      method
    });
  };
}
