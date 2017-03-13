/**
 * Get the value of a querystring parameter
 * @param  {String} param The field to get the value of
 * @param  {String} url   The URL to get the value from (optional)
 * @return {String}       The param value
 */
export default function getQueryStringParam(param, url) {
  let href = url ? url : window.location.href;
  let reg = new RegExp( '[?&]' + param + '=([^&#]*)', 'i' );
  let string = reg.exec(href);
  return string ? decodeURIComponent(string[1]) : null;
}
