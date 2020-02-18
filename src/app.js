const { existsSync } = require('fs');
const Logger = require('./structures/Logger');
const Server = require('./structures/Server');
const utils = require('./util');

const logger = new Logger('Master');
if (!existsSync(utils.getArbitrayPath('config.json'))) {
  logger.warn(`Missing "${logger.colors.yellow('config.json')}" file in path ${utils.getArbitrayPath('config.json')}`);
  process.exit(1);
}

const config = require('./config.json');
const server = new Server(config);

logger.info('Now running the server...');
server.launch();

process.on('unhandledRejection', reason => logger.error('Received "unhandledRejection" error', reason || new Error('No reason provided')));
process.on('uncaughtException', ex => logger.error('Received "uncaughtException" error', ex));