export function get_scores() {
  const request = new XMLHttpRequest();
  request.open('GET', '/api/score', false);  // `false` makes the request synchronous
  request.send(null);

  if (request.status === 200) {
    return request.responseText;
  }
}

export function post_score(name, score) {
  const request = new XMLHttpRequest();
  request.open('POST', `/api/score/${name}/${score}`, false);  // `false` makes the request synchronous
  request.send(null);
}