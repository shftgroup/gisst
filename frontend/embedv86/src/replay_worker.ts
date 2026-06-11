import {BlockIndex} from './blockindex';
import {bytes_to_uuid,uuid_to_bytes,blobToDataURL,dataURLToBlob} from './utils';
import {Evt,ReplayEvent,Checkpoint} from './v86replay';
import {WorkerCommand,EncodedCheckpoint} from './worker_protocol';
const STATE_INDEX_INFO_LEN = 3;
const STATE_INFO_BLOCK_START = 16;

const BLOCK_SIZE=256;
const SUPERBLOCK_SIZE=256;

class ReplayData {
  block_index:BlockIndex;
  superblock_index:BlockIndex;
  id:string="";
  version:number=0;
  constructor(id:string, version:number, block_index:BlockIndex, superblock_index:BlockIndex) {
    this.id = id;
    this.version = version;
    this.block_index = block_index;
    this.superblock_index = superblock_index;
  }
  async restore_checkpoint(header_info:Uint8Array, superblock_seq:Uint32Array):Promise<ArrayBuffer> {
    // if there was already a pending decode, drop it
    const block_byte_size = this.block_index.object_size;
    const superblock_byte_size = (this.superblock_index.object_size/4) * block_byte_size;
    /* TODO reuse this allocation across calls */
    const header_info_length = header_info.length;
    const full_size = (new Uint32Array(header_info.buffer,0,4))[2];
    const buffer = new ArrayBuffer(full_size);
    const state = new Uint8Array(buffer);
    // TODO: maybe use jsonpatch / micropatch to patch this info block from the previous one, but this means restores need to be sequential
    state.set(header_info, 0);
    for (let i = 0; i < superblock_seq.length; i++) {
      const superblock_idx = superblock_seq[i];
      const superblock_bytes = this.superblock_index.get(superblock_idx);
      const superblock = new Uint32Array(superblock_bytes.buffer, superblock_bytes.byteOffset, superblock_bytes.byteLength / 4);
      const superblock_sz = superblock.length;
      for (let j = 0; j < superblock_sz; j++) {
        const block_idx = superblock[j];
        const byte_offset = header_info_length + i * superblock_byte_size + j * block_byte_size;
        if (byte_offset >= state.byteLength) {
          break;
        }
        const block = this.block_index.get(block_idx);
        if (byte_offset + block_byte_size > state.byteLength) {
          state.set(block.subarray(0, state.byteLength - byte_offset), byte_offset);
          break;
        } else {
          state.set(block, byte_offset);
        }
      }
    }
    return buffer;
  }
  async encode_checkpoint(time:number, state:Uint8Array, which:number): Promise<EncodedCheckpoint> {
    // write to file
    const header_block = new Int32Array(state.buffer, state.byteOffset, 4);
    const info_block_len = header_block[STATE_INDEX_INFO_LEN];
    // TODO: maybe use jsondiff / microdiff to take a diff of this info block from the last one
    const info_block_buffer = state.slice(0, STATE_INFO_BLOCK_START + info_block_len);
    state = state.subarray(STATE_INFO_BLOCK_START + info_block_len);
    const state_size = state.length;
    const block_byte_size = this.block_index.object_size;
    const superblock_block_count = this.superblock_index.object_size/4;
    const superblock_byte_size = superblock_block_count*block_byte_size;
    const superblock_count = Math.ceil(state_size / (this.block_index.object_size*superblock_block_count));
    /* TODO reuse these allocations across calls */
    const superblock_seq = new Uint32Array(superblock_count);
    const superblock_buf = new Uint32Array(new ArrayBuffer(this.superblock_index.object_size));
    const block_buf = new Uint8Array(block_byte_size);
    const new_blocks = [];
    const new_superblocks = [];
    for (let i = 0; i < superblock_count; i++) {
      const superblock_offset = i * superblock_byte_size;
      for (let j = 0; j < superblock_block_count; j++) {
        const block_start = superblock_offset + j * block_byte_size;
        const block_end = Math.min(block_start + block_byte_size, state_size);
        if (block_start >= state_size) {
          superblock_buf[j] = 0;
        } else {
          let result;
          if(block_start + block_byte_size > state_size) {
            block_buf.subarray(state_size - block_start).fill(0);
            block_buf.set(state.subarray(block_start, block_end));
            result = await this.block_index.insert(block_buf, time);
          } else {
            result = await this.block_index.insert(state.slice(block_start, block_end), time);
          }
          if (result.is_new) {
            // new block; output or add to a list
            new_blocks.push(result.index);
          }
          superblock_buf[j] = result.index;
        }
      }
      const super_result = await this.superblock_index.insert(new Uint8Array(superblock_buf.buffer), time);
      if (super_result.is_new) {
        // new superblock; output or add to a list
        new_superblocks.push(super_result.index);
      }
      superblock_seq[i] = super_result.index;
    }
    return {
      header_info:info_block_buffer,
      superblock_seq,
      new_blocks,
      new_superblocks,
      which
    };
  }
  async serialize(events:ReplayEvent[], checkpoints:Checkpoint[]):Promise<ArrayBuffer> {
    // header data
    const frame_count = events.length;
    const checkpoint_count = checkpoints.length;
    const last_check = checkpoints[checkpoints.length-1];
    // magic+metadata+frame count+checkpoint count+(frame count * max event size)+(checkpoint count * size of last checkpoint)+size of deduped blocks and superblocks
    const size_max = 4+32+4+4+(frame_count*(1+8+4*8))+(checkpoint_count*(8+4+4+last_check.thumbnail.length+4+last_check.header_info.byteLength+4+4+4+last_check.superblock_seq.length*4))+this.block_index.length()*this.block_index.object_size+this.superblock_index.length()*this.superblock_index.object_size+this.block_index.length()*4+this.superblock_index.length()*4;
    const ret = new ArrayBuffer(size_max, {maxByteLength:size_max});
    const view = new DataView(ret);
    // ASCII "VRPL" backwards ("LPRV") so it shows up as "VRPL" in binary
    let x = 0;
    view.setUint32(x,0x4C505256,true);
    x += 4;
    // metadata: 16 bytes UUID; reserve the rest for later.
    {
      const id_bytes = uuid_to_bytes(this.id);
      const dst = new Uint8Array(ret, x, 16);
      dst.set(id_bytes);
      x += 16;
    }
    // write version
    view.setUint8(x, this.version);
    x += 1;
    // empty 15 bytes
    x += 15;
    // counts
    view.setUint32(x,frame_count,true);
    x += 4;
    view.setUint32(x,checkpoint_count,true);
    x += 4;
    // these will be written as we go
    for(const evt of events) {
      view.setUint8(x,evt.code as number);
      x += 1;
      view.setBigUint64(x,BigInt(evt.when),true);
      x += 8;
      let val;
      switch(evt.code) {
        case Evt.KeyCode:
          view.setUint8(x,evt.value as number);
          x += 1;
          break;
        case Evt.MouseClick:
          val = <number[]>evt.value;
          view.setUint8(x,val[0]);
          view.setUint8(x+1,val[1]);
          view.setUint8(x+2,val[2]);
          x += 3;
          break;
        case Evt.MouseDelta:
        case Evt.MouseWheel:
          val = <number[]>evt.value;
          view.setFloat64(x,val[0],true);
          if (Number.isNaN(view.getFloat64(x,true))) {
            console.error("uh oh");
            throw "no way";
          }
          view.setFloat64(x+8,val[1],true);
          x += 8*2;
          break;
        case Evt.MouseAbsolute:
          val = <number[]>evt.value;
          view.setFloat64(x,val[0],true);
          view.setFloat64(x+8,val[1],true);
          view.setFloat64(x+16,val[2],true);
          view.setFloat64(x+24,val[3],true);
          x += 8*4;
          break;
        default:
          throw "Unhandled event type";
      }
    }
    // these will be written as we go
    for(const check of checkpoints) {
      // the when
      view.setBigUint64(x,BigInt(check.when),true);
      x += 8;
      // the event index
      view.setUint32(x,check.event_index,true);
      x += 4;
      // the thumbnail; TODO decode base64
      const thumb_bytes = await dataURLToBlob(check.thumbnail).arrayBuffer();
      view.setUint32(x,thumb_bytes.byteLength,true);
      x += 4;
      {
        const dst = new Uint8Array(ret, x, thumb_bytes.byteLength);
        dst.set(new Uint8Array(thumb_bytes));
      }
      x += thumb_bytes.byteLength;
      // the state
      // write header size
      view.setUint32(x,check.header_info.byteLength,true);
      x += 4;
      // write header
      {
        const dst = new Uint8Array(ret, x, check.header_info.byteLength);
        // low budget idea: do json diff generation here from the last decoded info block
        dst.set(check.header_info);
        x += check.header_info.byteLength;
      }
      // write new block count and contents
      view.setUint32(x, check.new_blocks.length, true);
      x += 4;
      for (const block_idx of check.new_blocks) {
        const dst = new Uint8Array(ret, x, this.block_index.object_size);
        dst.set(this.block_index.get(block_idx));
        x += this.block_index.object_size;
      }
      // write new superblock count and contents
      view.setUint32(x, check.new_superblocks.length, true);
      x += 4;
      for (const superblock_idx of check.new_superblocks) {
        const dst = new Uint8Array(ret, x, this.superblock_index.object_size);
        dst.set(this.superblock_index.get(superblock_idx));
        x += this.superblock_index.object_size;
      }
      // Write superblock seq count and contents
      view.setUint32(x, check.superblock_seq.length, true);
      x += 4;
      {
        for (const superblock of check.superblock_seq) {
          view.setUint32(x, superblock, true);
          x += 4;
        }
      }
    }
    ret.resize(x);
    return ret;
  }
  static async deserialize(buf: ArrayBuffer): Promise<{worker_data:ReplayData, replay_data:{
    id:string,
    version:number,
    events:ReplayEvent[],
    checkpoints:Checkpoint[]
  }}> {
    const events = [];
    const checkpoints = [];
    const view = new DataView(buf);
    let x = 0;
    // magic, already validated
    x += 4;
    // metadata: 16 bytes UUID; reserve the rest for later.
    const id = bytes_to_uuid(new Uint8Array(buf, x, 16));
    x += 16;
    const version = view.getUint8(x);
    x += 1;
    // empty metadata bytes
    x += 15;
    const frame_count = view.getUint32(x, true);
    x += 4;
    const checkpoint_count = view.getUint32(x, true);
    x += 4;
    const r = new ReplayData(id, version, await BlockIndex.create(BLOCK_SIZE), await BlockIndex.create(SUPERBLOCK_SIZE*4));
    for (let i = 0; i < frame_count; i++) {
      const code = view.getUint8(x);
      x += 1;
      const when_b = view.getBigUint64(x, true);
      x += 8;
      if (when_b > BigInt(Number.MAX_SAFE_INTEGER)) {
        console.log(when_b);
        throw "When is too big";
      }
      const when = Number(when_b);
      if (BigInt(when) != when_b) {
        console.log(when, when_b);
        throw "When didn't match";
      }
      let value: object | number;
      switch (code) {
        case Evt.KeyCode:
          value = view.getUint8(x);
          x += 1;
          break;
        case Evt.MouseClick: {
          const a = view.getUint8(x);
          const b = view.getUint8(x + 1);
          const c = view.getUint8(x + 2);
          x += 3;
          value = [a, b, c];
          break;
        }
        case Evt.MouseDelta:
        case Evt.MouseWheel: {
          const mx = view.getFloat64(x, true);
          const my = view.getFloat64(x + 8, true);
          value = [mx, my];
          x += 8 * 2;
          break;
        }
        case Evt.MouseAbsolute: {
          const mx = view.getFloat64(x, true);
          const my = view.getFloat64(x + 8, true);
          const w = view.getFloat64(x + 16, true);
          const h = view.getFloat64(x + 24, true);
          value = [mx, my, w, h];
          x += 8 * 4;
          break;
        }
        default:
          throw "Unhandled event type";
      }
      events.push(new ReplayEvent(when, code, value));
    }
    // these will be read as we go
    for (let i = 0; i < checkpoint_count; i++) {
      const when_b = view.getBigUint64(x, true);
      x += 8;
      if (when_b > BigInt(Number.MAX_SAFE_INTEGER)) {
        console.log(when_b);
        throw "When is too big";
      }
      const when = Number(when_b);
      if (BigInt(when) != when_b) {
        console.log(when, when_b);
        throw "When didn't match";
      }
      const event_index = view.getUint32(x, true);
      x += 4;
      const name = "replay"+id+"-state" + i.toString();
      const thumb_len = view.getUint32(x, true);
      x += 4;
      const thumb_bytes = new Uint8Array(thumb_len);
      thumb_bytes.set(new Uint8Array(buf, x, thumb_len));
      const thumb = import.meta.vitest ? "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==" : await blobToDataURL(new Blob([thumb_bytes], { type: "image/png" }));
      x += thumb_len;
      // version 0 handled in LegacyReplay
      if (version == 1) {
        /* already state stream, read new blocks, new superblocks, superblock seq */
        const info_len = view.getUint32(x, true);
        x += 4;
        const info = new Uint8Array(buf, x, info_len);
        // low budget idea: do json diff application here from the last decoded info block
        x += info_len;
        const new_blocks = [];
        const new_block_count = view.getUint32(x, true);
        x += 4;
        for (let i = 0; i < new_block_count; i++) {
          const block = new Uint8Array(buf, x, r.block_index.object_size);
          new_blocks.push(r.block_index.length());
          await r.block_index.insert_exact(block.slice(), r.block_index.length(), when);
          x += r.block_index.object_size;
        }
        const new_superblocks = [];
        const new_superblock_count = view.getUint32(x, true);
        x += 4;
        for (let i = 0; i < new_superblock_count; i++) {
          const superblock = new Uint8Array(buf, x, r.superblock_index.object_size);
          new_superblocks.push(r.superblock_index.length());
          await r.superblock_index.insert_exact(superblock.slice(), r.superblock_index.length(), when);
          x += r.superblock_index.object_size;
        }
        const superblock_seq_len = view.getUint32(x, true);
        x += 4;
        const superblock_seq = new Uint32Array(superblock_seq_len);
        for (let i = 0; i < superblock_seq_len; i++) {
          superblock_seq[i] = view.getUint32(x, true);
          x += 4;
        }
        checkpoints.push(new Checkpoint(when, name, event_index, info.slice(), new_blocks, new_superblocks, superblock_seq.slice(), thumb));
      }
    }
    return {worker_data:r, replay_data:{id:r.id, version:r.version, events, checkpoints}};
  }
}
let replay;
const queue:WorkerCommand[] = [];
let busy = false;
if (!import.meta.vitest) {
onmessage = function (e) {
  queue.push(e.data as WorkerCommand);
  pumpQueue();
};
}
function pumpQueue() {
  if (!busy && queue.length > 0) {
    processCommand(queue.shift()!);
  } else if (busy) {
    setTimeout(pumpQueue, 0);
  }
}
function finishCommand() {
  busy = false;
  console.log("finished");
  if (queue.length) {
    pumpQueue();
  }
}
function processCommand(data:WorkerCommand) {
  busy = true;
  console.log("start cmd",data);
  switch (data.type) {
    case "init":{
      Promise.all([
        BlockIndex.create(BLOCK_SIZE),
        BlockIndex.create(SUPERBLOCK_SIZE*4)]).then(([bi,sbi]) => {
        replay = new ReplayData(data.args.id, data.args.version, bi, sbi);
        postMessage({type:"initialized"});
        finishCommand();
      });
      break;
    }
    case "drop_checkpoints":{
      const when = data.args.when;
      replay!.block_index.remove_after(when);
      replay!.superblock_index.remove_after(when);
        finishCommand();
      break;
    }
    case "serialize":{
      const events = data.args.events;
      const checkpoints = data.args.checkpoints;
      replay!.serialize(events,checkpoints).then((ab:ArrayBuffer) => {
        postMessage({type:"serialize",args:{result:ab}},{transfer:[ab]});
        finishCommand();
      });
      break;}
    case "deserialize": {
      const buffer = data.args.buffer;
      ReplayData.deserialize(buffer).then((result) => {
        replay = result.worker_data;
        postMessage({type:"deserialize",args:result.replay_data});
        finishCommand();
      });
      break;
    }
    case "encode": {
      const time = data.args.time;
      const state = data.args.state;
      const which = data.args.which;
      replay!.encode_checkpoint(time, state, which).then((result:EncodedCheckpoint) => {
        postMessage({type:"checkpoint", args:result},{transfer:[result.header_info.buffer,result.superblock_seq.buffer]});
        finishCommand();
      });
      break;
    }
    case "decode": {
      const header_info = data.args.header_info;
      const superblock_seq = data.args.superblock_seq;
      const which = data.args.which;
      replay!.restore_checkpoint(header_info, superblock_seq).then((result:ArrayBuffer) => {
        postMessage({type:"decode_checkpoint", args:{which,result}},{transfer:[result]});
        finishCommand();
      });
      break;
    }
    default:
      throw "Unrecognized message";
  }
};
setTimeout(pumpQueue, 0);
if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;
  it("encode", async () => {
    const replay = new ReplayData("123456789abcdef123456789abcdef", 1, await BlockIndex.create(BLOCK_SIZE), await BlockIndex.create(SUPERBLOCK_SIZE*4));
    const header = [
      0,0,0,0,
      0,0,0,0,
      0,0,0,0,
      4,0,0,0,
      1,2,3,4,
    ];
    const fake_state = new Uint8Array(16+4+256*256*2);
    fake_state.set(header);
    /* all 0s except the last one */
    fake_state[fake_state.length-1] = 1;
    const cp1 = await replay.encode_checkpoint(0, fake_state.slice(), 0);
    expect(cp1.header_info.length).toBe(20);
    expect(replay.block_index.length()).toBe(2);
    expect(replay.superblock_index.length()).toBe(2);
    expect(cp1.superblock_seq.length).toBe(2);
    expect(cp1.superblock_seq).toEqual(Uint32Array.from([0,1]));
    fake_state[fake_state.length-1] = 2;
    const cp2 = await replay.encode_checkpoint(1, fake_state, 1);
    expect(replay.block_index.length()).toBe(3);
    expect(replay.superblock_index.length()).toBe(3);
    expect(cp2.superblock_seq.length).toBe(2);
    expect(cp2.superblock_seq).toEqual(Uint32Array.from([0,2]));
  });
  it("decode", async () => {
    const replay = new ReplayData("123456789abcdef123456789abcdef", 1, await BlockIndex.create(BLOCK_SIZE), await BlockIndex.create(SUPERBLOCK_SIZE*4));
    const header = [
      0,0,0,0,
      0,0,0,0,
      20,0,2,0,
      4,0,0,0,
      1,2,3,4,
    ];
    const fake_state = new Uint8Array(16+256*256*2);
    fake_state.set(header);
    /* all 0s except the last byte */
    fake_state[fake_state.length-1] = 1;
    const cp1 = await replay.encode_checkpoint(0, fake_state.slice(), 0);
    const state_at_cp1 = fake_state.slice().buffer;
    fake_state[fake_state.length-1] = 2;
    const cp2 = await replay.encode_checkpoint(1, fake_state.slice(), 1);
    const state_at_cp2 = fake_state.slice().buffer;
    fake_state[fake_state.length-1] = 4;
    expect(replay.block_index.length()).toBe(3);
    expect(replay.superblock_index.length()).toBe(3);
    const out_state_1 = await replay.restore_checkpoint(cp1.header_info, cp1.superblock_seq);
    expect(out_state_1).toEqual(state_at_cp1);
    const out_state_2 = await replay.restore_checkpoint(cp2.header_info, cp2.superblock_seq);
    expect(out_state_2).toEqual(state_at_cp2);
  });
  it("roundtrip", async () => {
    const IMG_STR = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
    const replay = new ReplayData("123456789abcdef123456789abcdef", 1, await BlockIndex.create(BLOCK_SIZE), await BlockIndex.create(SUPERBLOCK_SIZE*4));
    const header = [
      0,0,0,0,
      0,0,0,0,
      20,0,2,0,
      4,0,0,0,
      1,2,3,4,
    ];
    const fake_state = new Uint8Array(16+4+256*256*2);
    fake_state.set(header);
    /* all 0s except the last one */
    fake_state[fake_state.length-1] = 1;
    const cp1_data = await replay.encode_checkpoint(0, fake_state.slice(), 0);
    const cp1 = new Checkpoint(0, "replay0-state0", 0, cp1_data.header_info, cp1_data.new_blocks, cp1_data.new_superblocks, cp1_data.superblock_seq, IMG_STR);
    fake_state[fake_state.length-1] = 2;
    const cp2_data = await replay.encode_checkpoint(1, fake_state.slice(), 1);
    const cp2 = new Checkpoint(1, "replay0-state1", 0, cp2_data.header_info, cp2_data.new_blocks, cp2_data.new_superblocks, cp2_data.superblock_seq, IMG_STR);
    const ser = await replay.serialize([], [cp1,cp2]);
    const replay2_data = await ReplayData.deserialize(ser);
    const replay2 = replay2_data.replay_data;
    const replay2_rdata = replay2_data.worker_data;

    expect(cp1.header_info).toEqual(replay2.checkpoints[0].header_info);
    expect(cp1.superblock_seq).toEqual(replay2.checkpoints[0].superblock_seq);
    expect(cp1.new_blocks).toEqual(replay2.checkpoints[0].new_blocks);
    expect(cp1.new_superblocks).toEqual(replay2.checkpoints[0].new_superblocks);
    expect(cp2.header_info).toEqual(replay2.checkpoints[1].header_info);
    expect(cp2.superblock_seq).toEqual(replay2.checkpoints[1].superblock_seq);
    expect(cp2.new_blocks).toEqual(replay2.checkpoints[1].new_blocks);
    expect(cp2.new_superblocks).toEqual(replay2.checkpoints[1].new_superblocks);
    expect(replay.block_index.get(0)).toEqual(replay2_rdata.block_index.get(0));
    expect(replay.block_index.get(1)).toEqual(replay2_rdata.block_index.get(1));
    expect(replay.block_index.get(2)).toEqual(replay2_rdata.block_index.get(2));
    expect(replay.superblock_index.get(0)).toEqual(replay2_rdata.superblock_index.get(0));
    expect(replay.superblock_index.get(1)).toEqual(replay2_rdata.superblock_index.get(1));
  });
}
