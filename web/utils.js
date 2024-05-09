export function nextEvent(target, name, predicate) {
  return new Promise(resolve => {
    target.addEventListener(name, function l(ev) {
      if (!predicate(ev)) return;
      target.removeEventListener(name, l);
      resolve(ev);
    });
  });
}
