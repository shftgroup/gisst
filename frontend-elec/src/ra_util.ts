enum RASTATE_BLOCK {
  MEM="MEM ",
  ACHV="ACHV",
  RPLY="RPLY",
  END="END "
}
const RASTATE_MAGIC="RASTATE";

export interface ReplayInfo {
  id:string;
  length:number;
  data:Uint8Array;
}

export function replay_of(data:ArrayBuffer):ReplayInfo {
  let magic = data.subarray(0,7);
  if(!magic.every((x,j) => x == RASTATE_MAGIC.charCodeAt(j)) || data[7] != 1) {
    console.log(Array.from(magic).map((x) => String.fromCharCode(x)), data[7]);
    throw "Not an RASTATE1 format file";
  }
  let i = 8;
  while(i < data.length) {
    // fetch header
    let block_size = (new DataView(data.buffer,i+4,i+8)).getUint32(0,true);
    let marker = data.subarray(i,i+4);
    i += 8;
    // check header contents
    if(marker.every((x,j)=>x==RASTATE_BLOCK.RPLY.charCodeAt(j))) {
      let header = new DataView(data.buffer,i,i+(7*4));
      let loaded_len = header.getUint32(0*4,true);
      // let magic = header.getUint32(1*4,true);
      // let vsn = header.getUint32(2*4,true);
      // let content_crc = header.getUint32(3*4,true);
      // let state_size = header.getUint32(4*4,true);
      let identifier = header.getBigInt64(5*4,true);
      return {id:identifier.toString(), length:loaded_len, data:data.subarray(i+(7*4), i+block_size)};
    } else {
      console.log("Skip state block "+Array.from(marker).map((x)=>String.fromCharCode(x)).join(""));
    }
    // align block to 8-byte boundary
    i += (block_size + 7) & ~7;
  }
  return null;
}
