const { promises: fs } = require('fs');
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
 * @param {number} [seed=16] The length of the string
 */
const generate = (seed) => randomBytes(seed || 16).toString('hex');

/**
 * If the OS is running Node.js 10 or higher
 */
const isNode10 = () => {
  const version = process.version.split('.')[0].replace('v', '');
  return Number(version) < 10;
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

module.exports = {
  getArbitrayPath,
  dateformat,
  generate,
  isNode10,
  sleep,
  sep
};