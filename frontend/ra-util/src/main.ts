enum RASTATE_BLOCK {
  MEM="MEM ",
  ACHV="ACHV",
  RPLY="RPLY",
  END="END "
}
const RASTATE_MAGIC="RASTATE";
//const RZIP_MAGIC="#RZIPv1#";

export interface ReplayInfo {
  id:string;
  data:Uint8Array;
}
export function replay_info(raw_data:Uint8Array):ReplayInfo {
  const data = raw_data.buffer;
  const header = new DataView(data,raw_data.byteOffset,6*4);
  // let magic = header.getUint32(0*4,true);
  // let vsn = header.getUint32(1*4,true);
  // let content_crc = header.getUint32(2*4,true);
  // let state_size = header.getUint32(3*4,true);
  const identifier = header.getBigInt64(4*4,true);
  return {id:identifier.toString(), data:raw_data.subarray(6*4,raw_data.length)};

}
export function replay_of_state(raw_bytes:Uint8Array):ReplayInfo|null {
  const data = raw_bytes.buffer;
  const magic = new Uint8Array(data, 0, 7);
  if(!magic.every((x,j) => x == RASTATE_MAGIC.charCodeAt(j)) || raw_bytes[7] != 1) {
    console.log(Array.from(magic).map((x) => String.fromCharCode(x)), raw_bytes[7]);
    throw new Error("Not an RASTATE1 format file");
  }
  let i = 8;
  while(i < data.byteLength) {
    // fetch header
    const block_size = (new DataView(data,i+4,4)).getUint32(0,true);
    const marker = new Uint8Array(data, i, 4);
    console.log(marker,block_size);
    i += 8;
    // check header contents
    if(marker.every((x,j)=>x==RASTATE_BLOCK.RPLY.charCodeAt(j))) {
      return replay_info(new Uint8Array(data, i+4, block_size-4));
    } else {
      console.log("Skip state block "+Array.from(marker).map((x)=>String.fromCharCode(x)).join(""));
    }
    // align block to 8-byte boundary
    i += (block_size + 7) & ~7;
  }
  return null;
}
