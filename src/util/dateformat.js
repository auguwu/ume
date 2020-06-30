// Credit: https://github.com/felixge/node-dateformat

class Dateformat {
  /**
   * Creates a new instance of the `Dateformat` class
   * @param {Date | number} date The date to retrive from
   */
  constructor(date) {
    this.settings = {
      utc: false,
      gmt: false
    };
    this.masks = {
      'default':               'ddd mmm dd yyyy HH:MM:ss',
      'shortDate':             'm/d/yy',
      'mediumDate':            'mmm d, yyyy',
      'longDate':              'mmmm d, yyyy',
      'fullDate':              'dddd, mmmm d, yyyy',
      'shortTime':             'h:MM TT',
      'mediumTime':            'h:MM:ss TT',
      'longTime':              'h:MM:ss TT Z',
      'isoDate':               'yyyy-mm-dd',
      'isoTime':               'HH:MM:ss',
      'isoDateTime':           'yyyy-mm-dd\'T\'HH:MM:sso',
      'isoUtcDateTime':        'UTC:yyyy-mm-dd\'T\'HH:MM:ss\'Z\'',
      'expiresHeaderFormat':   'ddd, dd mmm yyyy HH:MM:ss Z'
    };
    this.i18n = {
      dayNames: [
        'Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat',
        'Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'
      ],
      monthNames: [
        'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
        'January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'
      ],
      timeNames: [
        'a', 'p', 'am', 'pm', 'A', 'P', 'AM', 'PM'
      ]      
    };

    this.date = date instanceof Date ? date : new Date(date);
    this.token = /d{1,4}|m{1,4}|yy(?:yy)?|([HhMsTt])\1?|[LloSZWN]|"[^"]*"|'[^']*'/g;
    this.timezone = /\b(?:[PMCEA][SDP]T|(?:Pacific|Mountain|Central|Eastern|Atlantic) (?:Standard|Daylight|Prevailing) Time|(?:GMT|UTC)(?:[-+]\d{4})?)\b/g;
    this.timezoneClip = /[^-+\dA-Z]/g;
  }

  /**
   * @param {any} value The value
   * @param {number} length The length
   * @returns {string}
   */
  pad(value, length) {
    value = String(value);
    length = length || 2;

    while (value.length < length) value = `0${value}`;
    return value;
  }

  /**
   * Gets the week of the month
   */
  getWeek() {
    const target = new Date(this.date.getFullYear(), this.date.getMonth(), this.date.getDate());
    const thurs = (target.getDate() - ((target.getDay() + 6) % 7) + 3);
    target.setDate(thurs);

    const first = new Date(target.getFullYear(), 0, 4);
    first.setDate(first.getDate() - ((target.getDay() + 6) % 7) + 3);

    const diff = (target - first) / (86400000 * 7);
    return 1 + Math.floor(diff);
  }

  /**
   * Gets the day of the week
   */
  getDayOfWeek() {
    let dow = this.date.getDay();
    if (dow === 0) dow = 7;

    return dow;
  }

  /**
   * Returns a `kind of` solution
   * @param {any} val The value itself
   */
  getKindOf(value) {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value !== 'object') return typeof value;
    if (Array.isArray(value)) return 'array';

    return {}.toString.call(value).slice(8, -2).toLowerCase();
  }

  /**
   * Converts this instance to a String interpole
   * @param {string} mask The mask to use
   */
  toString(mask) {
    // If there is a mask and that mask doesn't exist
    mask = String(this.masks[mask] || mask || this.masks.default);

    const sliced = mask.slice(0, 4);
    if (sliced === 'UTC:' || sliced === 'GMT:') {
      const s = mask.slice(4);
      this.settings.utc = true;
      if (sliced === 'GMT:') this.settings.gmt = true;
    }

    const replacer = this.settings.utc ? 'getUTC' : 'get';
    const dict = {
      d: this.date[`${replacer}Date`](),
      D: this.date[`${replacer}Day`](),
      m: this.date[`${replacer}Month`](),
      y: this.date[`${replacer}FullYear`](),
      H: this.date[`${replacer}Hours`](),
      M: this.date[`${replacer}Minutes`](),
      s: this.date[`${replacer}Seconds`](),
      L: this.date[`${replacer}Milliseconds`](),
      o: this.settings.utc ? 0 : this.date.getTimezoneOffset(),
      W: this.getWeek(),
      N: this.getDayOfWeek()
    };

    const flags = {
      d: dict.d,
      dd: this.pad(dict.d),
      ddd: this.i18n.dayNames[dict.D],
      dddd: this.i18n.dayNames[dict.D + 7],
      m: dict.m + 1,
      mm: this.pad(dict.m + 1),
      mmm: this.i18n.monthNames[dict.m],
      mmmm: this.i18n.monthNames[dict.m + 12],
      yy: String(dict.y).slice(2),
      yyyy: dict.y,
      h: dict.H % 12 || 12,
      hh: this.pad(dict.H % 12 || 2),
      H: dict.H,
      HH: this.pad(dict.H),
      M: dict.M,
      MM: this.pad(dict.M),
      s: dict.s,
      ss: this.pad(dict.s),
      l: this.pad(dict.L, 3),
      L: this.pad(Math.round(dict.L / 10)),
      t: dict.H < 12 ? this.i18n.timeNames[0] : this.i18n.timeNames[1],
      tt: dict.H < 12 ? this.i18n.timeNames[2] : this.i18n.timeNames[3],
      T: dict.H < 12 ? this.i18n.timeNames[4] : this.i18n.timeNames[5],
      TT: dict.H < 12 ? this.i18n.timeNames[6] : this.i18n.timeNames[7],
      Z: this.settings.gmt ? 'GMT' : this.settings.utc ? 'UTC' : (String(this.date).match(this.timezone) || ['']).pop().replace(this.timezoneClip, ''),
      o: (dict.o > 0 ? '-' : '+') + this.pad(Math.floor(Math.abs(dict.o) / 60) * 100 + Math.abs(dict.o) % 60, 4),
      S: ['th', 'st', 'nd', 'rd'][dict.d % 10 > 3 ? 0 : (dict.d % 100 - dict.d % 10 != 10) * dict.d % 10],
      W: dict.W,
      N: dict.N
    };

    return mask.replace(this.token, (match) => {
      if (match in flags) return flags[match];
      return match.slice(1, match.length - 1);
    });
  }
}

/**
 * Manipulate a date to whatever you want
 * @param {Date | number} date The date to retrive from
 * @param {string} mask The mask to use
 * @returns The Time frame
 */
module.exports = (date, mask) => {
  const format = new Dateformat(date);
  return format.toString(mask);
};