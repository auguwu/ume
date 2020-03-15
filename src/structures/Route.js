const { Collection } = require('@augu/immutable');

class Route {
  /**
   * Creates a new `Route` instance
   * @param {string} route The route to use
   * @param {string} method The method to use
   * @param {(this: import('./Server'), req: import('express').Request, res: import('express').Response) => Promise<void>} callee The function to run the route
   */
  constructor(route, method, callee) {
    this.callee = callee;
    this.method = method;
    this.route = route;
  }
}

class Router {
  /**
   * Creates a new `Router` instance
   * @param {string} route The route to use
   */
  constructor(route) {
    /**
     * All of the routes
     * @type {Collection<Route>}
     */
    this.routes = new Collection();
    this.route = route;
  }

  /**
   * Adds a route
   * @param {Route} route The route to add
   */
  addRoute(route) {
    this.routes.set(route.route, route);
    return this;
  }
}

module.exports = {
  Router,
  Route
};