const { colors, hex } = require('leeks.js');
const { inspect } = require('util');

const LogLevel = {
  INFO: 0,
  WARN: 1,
  ERROR: 2,
  REQUEST: 3,
  DATABASE: 4
};

const Months = {
  0: 'Jan',
  1: 'Feb',
  2: 'Mar',
  3: 'Apr',
  4: 'May',
  5: 'Jun',
  6: 'Jul',
  7: 'Aug',
  8: 'Sept',
  9: 'Oct',
  10: 'Nov',
  11: 'Dec'
};

module.exports = class Logger {
  /**
   * Creates a new instance of the Logger class
   * @param {string} name The logger name
   */
  constructor(name) {
    this.colors = colors;
    this.name = name;
  }

  /**
   * Gets the current date
   */
  getDate() {
    const now = new Date();

    const year = now.getFullYear();
    const month = Months[now.getMonth()];
    const day = now.getDate();
    const hours = `0${now.getHours()}`.slice(-2);
    const minutes = `0${now.getMinutes()}`.slice(-2);
    const seconds = `0${now.getSeconds()}`.slice(-2);
    const ap = now.getHours() >= 12 ? 'PM' : 'AM';

    // Example: Feb 17th, 2020 16:29:55 PM
    return this.colors.gray(`[${month} ${day}, ${year} ${hours}:${minutes}:${seconds} ${ap}]`);
  }

  /**
   * Writes out anything to the terminal
   * @param {number} level The log level to use
   * @param {...(string|object)} message The message to print
   */
  write(level, ...message) {
    let lvlText;

    switch (level) {
      case LogLevel.INFO: {
        lvlText = this.colors.cyan(`[INFO/${process.pid}]`);
      } break;

      case LogLevel.WARN: {
        lvlText = this.colors.yellow(`[WARN/${process.pid}]`);
      } break;

      case LogLevel.ERROR: {
        lvlText = this.colors.red(`[ERROR/${process.pid}]`);
      } break;

      case LogLevel.REQUEST: {
        lvlText = hex('#B58183', `[REQUEST/${process.pid}]`);
      } break;

      case LogLevel.DATABASE: {
        lvlText = hex('#589636', `[DATABASE/${process.pid}]`);
      } break;
    }

    const name = hex('#A3919C', `[${this.name}]`);
    const msg = message.map(s =>
      s instanceof Object ? inspect(s) : s  
    );

    process.stdout.write(`${this.getDate()} ${lvlText} ${name} => ${msg.join('\n')}\n`);
  }

  /**
   * Writes as the `INFO` level
   * @param  {...(string|object)} message The message instance
   */
  info(...message) {
    return this.write(LogLevel.INFO, ...message);
  }

  /**
   * Writes as the `WARN` level
   * @param  {...(string|object)} message The message instance
   */
  warn(...message) {
    return this.write(LogLevel.WARN, ...message);
  }

  /**
   * Writes as the `ERROR` level
   * @param  {...(string|object)} message The message instance
   */
  error(...message) {
    return this.write(LogLevel.ERROR, ...message);
  }

  /**
   * Writes as the `REQUEST` level
   * @param  {...(string|object)} message The message instance
   */
  request(...message) {
    return this.write(LogLevel.REQUEST, ...message);
  }

  /**
   * Writes as the `DATABASE` level
   * @param  {...(string|object)} message The message instance
   */
  database(...message) {
    return this.write(LogLevel.DATABASE, ...message);
  }
};