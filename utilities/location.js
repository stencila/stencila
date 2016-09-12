// Utilities related to `document.location`
module.exports = {

  /**
   * Get the parameters of the query (`search`) part of the location
   *
   * Based on http://stackoverflow.com/a/1099670/4625911
   */
  params: function () {
    var qs = document.location.search.split('+').join(' ');

    var params = {};
    var tokens;
    var re = /[?&]?([^=]+)=([^&]*)/g;

    while (true) {
      tokens = re.exec(qs);
      if (!tokens) break;
      params[decodeURIComponent(tokens[1])] = decodeURIComponent(tokens[2]);
    }

    return params;
  }

};
