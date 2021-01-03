const Server = require('../src/structures/Server');

describe('Server Features', () => {
  it('should be undefined if gc is disabled', () => {
    const server = new Server({
      environment: 'jest',
      masterKey: '',
      dbUrl: '',
      port: 6969,
      features: {}
    });

    expect(server.gc).toBeUndefined();
  });

  it('should be defined if gc is enabled', () => {
    const server = new Server({
      environment: 'jest',
      masterKey: '',
      dbUrl: '',
      port: 6969,
      features: {
        gc: 6480000
      }
    });

    expect(server.gc).toBeDefined();
  });
});
