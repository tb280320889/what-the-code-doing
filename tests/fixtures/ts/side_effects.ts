export function readFile(path: string) {
  return fs.readFileSync(path, "utf-8");
}

export async function fetchData(url: string) {
  return fetch(url);
}

export function log(msg: string) {
  console.log(msg);
}

export function storeLocal(key: string, value: string) {
  localStorage.setItem(key, value);
}
