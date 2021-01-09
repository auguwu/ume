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

import type { Middleware } from '../structures';
import onFinished from 'on-finished';
import leeks from 'leeks.js';

function getRequestDuration(start: [number, number]) {
  const difference = process.hrtime(start);
  return (difference[0] * 1e9 + difference[1]) / 1e6;
}

const mod: Middleware = function logging() {
  return (req, res, next) => {
    next(); // continue

    const start = process.hrtime();
    onFinished(res, (_, res) => {
      let color;

      // this is gonna get real messy...
      if (res.statusCode >= 500)
        color = leeks.colors.red;
      else if (res.statusCode >= 400)
        color = leeks.colors.yellow;
      else if (res.statusCode >= 300)
        color = leeks.colors.magenta;
      else if (res.statusCode >= 200)
        color = leeks.colors.green;
      else
        color = leeks.colors.grey;

      const time = color(`~${getRequestDuration(start)}ms`);
      this.logger.request(`"${res.statusCode} -> ${req.method.toUpperCase()} ${req.url}" | ${time}`);
    });
  };
};

export default mod;
