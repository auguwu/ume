const { randomBytes } = require('crypto');
const dateformat = require('./dateformat');

/**
 * Gets the seperator for the corresponded OS
 */
const sep = process.platform === 'win32' ? '\\' : '/';

/**
 * Gets an arbitray path with the current directory appended
 * @param  {...string} paths The paths to include
 */
const getArbitrayPath = (...paths) => `${process.cwd()}${sep}${paths.join(sep)}`;

/**
 * Gets a random generated string from `seed`
 */
const generate = () => randomBytes(3).toString('hex');

/**
 * If the OS is running Node.js 10 or higher
 * @param {string} [version] The version to check
 */
const isNode10 = (version) => {
  const ver = version ? version.split('.')[0] : process.version.split('.')[0].replace('v', '');
  const num = Number(ver);

  return num === 10 || num > 10;
};

/**
 * Pauses the event loop to run this macrotask.
 * 
 * All promises are a macrotask while a setTimeout/setInterval is a microtask;
 * This function pauses the callback loop but lets the synchronous code run
 * but it'll pause until the duration has passed.
 * 
 * @param {number} duration The amount of milliseconds to pause this loop
 * @returns {Promise<unknown>} Returns an unknown promise. It makes sure
 * that runtime will know that this is an empty fulfilled promise.
 */
const sleep = (duration) => new Promise(resolve => setTimeout(resolve, duration));

/**
 * Formats any byte-integers to a humanizable string
 * @param {number} bytes The number of bytes the file is
 */
const formatSize = (bytes) => {
  const kilo = bytes / 1024;
  const mega = kilo / 1024;
  const giga = mega / 1024;

  if (kilo < 1024) return `${kilo.toFixed(1)}KB`;
  if (kilo > 1024 && mega < 1024) return `${mega.toFixed(1)}MB`;
  else return `${giga.toFixed(1)}GB`;
};

module.exports = {
  getArbitrayPath,
  formatSize,
  dateformat,
  generate,
  isNode10,
  sleep,
  sep
};
