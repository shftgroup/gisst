export function memcmp(a:Uint8Array, b:Uint8Array) : boolean {
  const a64 = new BigUint64Array(a.buffer, a.byteOffset, a.byteLength/8);
  const b64 = new BigUint64Array(b.buffer, b.byteOffset, b.byteLength/8);
  const sz = a64.length;
  if (sz != b64.length) {
    return false;
  }
  for (let i = 0; i < sz; i++) {
    if (a64[i] != b64[i]) {
      return false;
    }
  }
  return true;
}
