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

import { inspect } from 'util';
import leeks from 'leeks.js';

// List of colors for the log levels
const Colors = {
  request: leeks.hex('#9B8FEF', '[Request]'),
  debug:   leeks.hex('#4F7CA3', '[Debug]'),
  error:   leeks.hex('#FF2255', '[Error]'),
  warn:    leeks.hex('#FFFFC5', '[Warn]'),
  info:    leeks.hex('#BCD9FF', '[Info]')
} as const;

type LogMessage = string | object | Error | any[] | Date | null | undefined | number | boolean;

enum LogLevel {
  Info = 'info',
  Warn = 'warn',
  Error = 'error',
  Debug = 'debug',
  Request = 'request'
}

export default class Logger {
  /** The logger's name */
  private name: string;

  /**
   * Creates a new [Logger] instance
   * @param name The name of the [Logger]
   */
  constructor(name: string) {
    this.name = name;
  }

  private _getDate() {
    const current = new Date();

    const hours = current.getHours();
    const minutes = current.getMinutes();
    const seconds = current.getSeconds();
    const month = current.getMonth() + 1;
    const day = current.getDate();
    const year = current.getFullYear();

    return leeks.colors.gray(`[${day}/${month}/${year} @ ${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}]`);
  }

  private _formatMessage(...messages: LogMessage[]): string {
    return messages.map<string>((message: unknown) => {
      if ([null, undefined].includes(<any> message)) return leeks.colors.cyan(message === null ? 'null' : 'undefined');
      if (message instanceof Array) return message.join('\n');
      if (message instanceof Date) return leeks.colors.cyan(message.toUTCString());
      if (message instanceof Error) {
        const e = [`${message.name}: ${message.message}`];
        const stack = message.stack ? message.stack.split('\n').map(s => s.trim()) : [];
        stack.shift();

        const all = stack.map(s => {
          if (/(.+(?=)|.+) ((.+):\d+|[:\d+])[)]/g.test(s)) return s.match(/(.+(?=)|.+) ((.+):\d+|[:\d+])[)]/g)![0];
          if (/(.+):\d+|[:\d+][)]/g.test(s)) return s.match(/(.+):\d+|[:\d+][)]/g)![0];

          return s;
        });

        e.push(...all.map(item => `   • "${item.replace('at', '').trim()}"`));
        return e.join('\n');
      }

      if (!['object', 'function', 'undefined', 'string'].includes(typeof message)) return leeks.colors.cyan(<any> message);
      if (typeof message === 'object') return inspect(message, { depth: null });

      return message as any;
    }).join('\n');
  }

  private write(level: LogLevel, ...messages: LogMessage[]) {
    const lvlText = Colors[level];
    if (lvlText === undefined) throw new TypeError(`Invalid log level '${level}'`);

    const message = this._formatMessage(...messages);
    const name = leeks.hex('#F4A4D2', `[${this.name}]`);
    const date = this._getDate();
    const stream = level === LogLevel.Error ? process.stderr : process.stdout;

    stream.write(`${date} ${name} ${lvlText}     ${message}\n`);
  }

  info(...messages: LogMessage[]) {
    return this.write(LogLevel.Info, ...messages);
  }

  warn(...messages: LogMessage[]) {
    return this.write(LogLevel.Warn, ...messages);
  }

  error(...messages: LogMessage[]) {
    return this.write(LogLevel.Error, ...messages);
  }

  debug(...messages: LogMessage[]) {
    if (process.env.NODE_ENV !== 'development') return;
    return this.write(LogLevel.Debug, ...messages);
  }

  request(...messages: LogMessage[]) {
    return this.write(LogLevel.Request, ...messages);
  }
}
