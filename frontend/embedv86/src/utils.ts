export function memcmp(a:Uint8Array, b:Uint8Array) : boolean {
  const a32 = new Uint32Array(a.buffer, a.byteOffset, a.byteLength/8);
  const b32 = new Uint32Array(b.buffer, b.byteOffset, b.byteLength/8);
  const sz = a32.length;
  if (sz != b32.length) {
    return false;
  }
  for (let i = 0; i < sz; i++) {
    if (a32[i] != b32[i]) {
      return false;
    }
  }
  return true;
}
