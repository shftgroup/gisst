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
export function bytes_to_uuid(buf:Uint8Array):string {
  // https://stackoverflow.com/a/50767210
  function bufferToHex (buffer:Uint8Array) {
    return [...buffer]
      .map (b => b.toString (16).padStart (2, "0"))
      .join ("");
  }
  // format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
  const s = bufferToHex(buf);
  return [s.slice(0,8),s.slice(8,12),s.slice(12,16),s.slice(16,20),s.slice(20,32)].join("-");
}
export function nonnull(obj:unknown):asserts obj {
  if(obj == null) {
    throw "Must be non-null";
  }
}
//https://stackoverflow.com/a/30407959
export function dataURLToBlob(dataurl:string) {
  const arr = dataurl.split(','), mime = arr[0]!.match(/:(.*?);/)![1],
    bstr = atob(arr[1]!);
  let n = bstr.length;
  const u8arr = new Uint8Array(n);
  while(n--){
    u8arr[n] = bstr.charCodeAt(n);
  }
  return new Blob([u8arr], {type:mime});
}
//https://stackoverflow.com/a/67551175
export function blobToDataURL(blob: Blob): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error);
    reader.onabort = () => reject(new Error("Read aborted"));
    reader.readAsDataURL(blob);
  });
}
export function uuid_to_bytes(s:string):Uint8Array {
  // format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
  const out = new Uint8Array(16);
  out[0] = parseInt(s.slice(0,2),16);
  out[1] = parseInt(s.slice(2,4),16);
  out[2] = parseInt(s.slice(4,6),16);
  out[3] = parseInt(s.slice(6,8),16);

  out[4] = parseInt(s.slice(9,11),16);
  out[5] = parseInt(s.slice(11,13),16);

  out[6] = parseInt(s.slice(14,16),16);
  out[7] = parseInt(s.slice(16,18),16);

  out[8] = parseInt(s.slice(19,21),16);
  out[9] = parseInt(s.slice(21,23),16);

  out[10] = parseInt(s.slice(24,26),16);
  out[11] = parseInt(s.slice(26,28),16);
  out[12] = parseInt(s.slice(28,30),16);
  out[13] = parseInt(s.slice(30,32),16);
  out[14] = parseInt(s.slice(32,34),16);
  out[15] = parseInt(s.slice(34,36),16);

  return out;
}
