const utils = require('../src/util');

describe('Utils', () => {
  it('should append "test" to the path', () => {
    const curr = process.cwd();
    const sep = utils.sep;

    expect(utils.getArbitrayPath('test')).toBeDefined();
    expect(utils.getArbitrayPath('test')).toStrictEqual(`${curr}${sep}test`);
  });

  it('should equal "815.0KB"', () => {
    expect(utils.formatSize(834560)).toBeDefined();
    expect(utils.formatSize(834560)).toStrictEqual('815.0KB');
  });

  describe('Node Versioning', () => {
    it('should be "false" if the version is "8.7.5"', () => {
      const version = '8.7.5';
      const value = utils.isNode10(version);

      expect(value).toBeDefined();
      expect(value).toBe(false);
    });

    it('should be "true" if the version is "10.7.2"', () => {
      const version = '10.7.2';
      const value = utils.isNode10(version);

      expect(value).toBeDefined();
      expect(value).toBe(true);
    });

    it('should be "true" if the version is "13.8.9"', () => {
      const version = '13.8.9';
      const value = utils.isNode10(version);

      expect(value).toBeDefined();
      expect(value).toBe(true);
    });
  });
});