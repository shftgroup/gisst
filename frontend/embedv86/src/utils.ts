export function memcmp(a:Uint8Array, b:Uint8Array) : boolean {
  const sz = a.length;
  if (sz != b.length) {
    return false;
  }
  for (let i = 0; i < sz; i++) {
    if (a[i] != b[i]) {
      return false;
    }
  }
  return true;
}
