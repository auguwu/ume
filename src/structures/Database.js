const { MongoClient } = require('mongodb');
const Logger = require('./Logger');

module.exports = class Database {
  /**
   * Creates a new instance of the `Database` class
   * @param {string} url The database URL
   */
  constructor(url) {
    this.url = url;
    this.logger = new Logger('Database');
    this.client = new MongoClient(url, {
      useUnifiedTopology: true,
      useNewUrlParser: true
    });
  }

  /**
   * Connects to a new database pool
   */
  async connect() {
    if (this.client.isConnected()) {
      this.logger.warn('Hm, it seems you are creating a new connection pool when one is established already!');
      return;
    }

    await this.client.connect();
    this.db = this.client.db('sharex');
    this.admin = this.db.admin();
    this.build = await this.admin.buildInfo();

    this.logger.database(`Connected to MongoDB with URI: ${this.url}`);
  }

  /**
   * Disposes the connection pool
   */
  async dispose() {
    if (!this.client.isConnected()) {
      this.logger.warn('Um, I don\'t think you should dispose a connection when there isn\'t one established...');
      return;
    }

    await this.client.close();
    this.db = null;
    this.admin = null;
    this.build = null;

    this.logger.database('Database connection was disposed');
  }

  /**
   * Gets the images collection
   * @returns {ImageCollection} The collection
   */
  get images() {
    return this.db.collection('images');
  }

  /**
   * Gets an image by it's UUID
   * @param {string} uuid The UUID
   */
  getImage(uuid) {
    return this.images.findOne({ uuid });
  }

  /**
   * Inserts a new image instance to the database
   * @param {Image} doc The document iself
   */
  addImage(doc) {
    this.images.insertOne(doc);
    return doc;
  }

  /**
   * Deletes an image instance by it's UUID
   * @param {string} uuid The UUID
   */
  delImage(uuid) {
    return this.images.deleteOne({ uuid });
  }
};

/**
 * @typedef {import('mongodb').Collection<Image>} ImageCollection The images collection
 * @typedef {object} Image
 * @prop {string} createdAt Formatted date of when the file was created
 * @prop {string} size The size of the file
 * @prop {string} path The path to the file
 * @prop {string} uuid The UUID of the image
 * @prop {"png" | "jpg" | "gif" | "webp"} ext The extension of the file
 */