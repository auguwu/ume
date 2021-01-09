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

import type { NextFunction, Request, Response } from 'express';
import { convertTime } from '../util';
import { Logger, Server } from '.';
import { Queue } from '@augu/collections';

interface RatelimitRecord {
  resetTime: Date;
  remaning: number;
  ip: string;
}

export default class RatelimitHandler {
  private _purgeTimeout: NodeJS.Timeout;
  private records: Queue<RatelimitRecord> = new Queue();
  private logger: Logger = new Logger('Ratelimits');
  private server: Server;

  constructor(server: Server) {
    const purgeAt = server.config.get<string | number>('ratelimits.purgeAt', '1 hour');

    this._purgeTimeout = setInterval(() => this.purge(), convertTime(purgeAt));
    this.server = server;
  }

  get middleware() {
    return (req: Request, res: Response, next: NextFunction) => this.handle(req, res, next);
  }

  dispose() {
    this.logger.warn('Disposing all records...');
    this.purge();

    clearInterval(this._purgeTimeout);
  }

  private _default(ip: string, limit: number): RatelimitRecord {
    const resetTime = new Date();
    resetTime.setMilliseconds(resetTime.getMilliseconds() + convertTime(this.server.config.get<string | number>('ratelimits.time', '1 hour')));

    return {
      resetTime,
      remaning: limit,
      ip
    };
  }

  private handle(req: Request, res: Response, next: NextFunction) {
    // Don't do ratelimits on localhost >W>
    if (req.ip === '::1') {
      return next();
    }

    this.logger.debug(`Polling ratelimits on ${req.ip}...`);
    const limit = this.server.config.get<number>('ratelimits.limit', 1000);
    const record = this.records.find(record =>
      record.ip === req.ip
    ) ?? this._default(req.ip, limit);

    record.remaning--;

    // Include the headers
    if (!res.headersSent) {
      res.setHeader('X-RateLimit-Remaining', record.remaning);
      res.setHeader('X-RateLimit-Limit', limit);
      res.setHeader('X-RateLimit-Reset', Math.ceil(record.resetTime.getTime() / 1000));

      // @ts-ignore It does exist but not in typings?
      res.setHeader('Date', new Date().toGMTString());
    }

    let hasDecrement = false;
    const decrement = () => {
      if (!hasDecrement) {
        hasDecrement = true;

        // @ts-ignore
        this.records.items = this.records.items.filter((value) => value.ip !== req.ip);
        this.records.push(record);
      }
    };

    res.on('finish', () => {
      if (res.statusCode >= 400) decrement();
      if (res.statusCode < 400) decrement();
    });

    res.on('close', () => {
      if (!res.writableEnded) decrement();
    });

    res.on('error', decrement);

    if (record.remaning === 0) {
      if (!res.headersSent) {
        res.setHeader('Retry-After', Math.ceil(convertTime(this.server.config.get<string | number>('ratelimits.time', '1 hour')) / 1000));
      }

      return res.status(429).json({
        message: `IP ${req.ip} is being ratelimited, try again later.`,
        code: 'RATELIMITED'
      });
    }

    next();
  }

  private purge() {
    this.logger.debug('Purging records...');

    const size = this.records.size();
    for (let i = size - 1; i >= 0; i--) this.records.remove(i);

    this.logger.debug(`Purged ${size} records from cache`);
  }
}
