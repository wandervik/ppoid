export function get_scores() {
  const request = new XMLHttpRequest();
  request.open('GET', 'http://api.ppoidgame.click/score', false);  // `false` makes the request synchronous
  request.send(null);

  if (request.status === 200) {
    return request.responseText;
  }
}

export function post_score(name, score) {
  const request = new XMLHttpRequest();
  request.open('POST', `http://api.ppoidgame.click/score/${name}/${score}`, false);
  request.send(null);
}
