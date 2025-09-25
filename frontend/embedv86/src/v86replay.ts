import {BlockIndex} from './blockindex';

export enum Evt {
  KeyCode = 0,
  MouseClick = 1,
  MouseDelta = 2,
  MouseAbsolute = 3,
  MouseWheel = 4
}
export const EvtNames:string[] = ["keyboard-code", "mouse-click", "mouse-delta", "mouse-absolute", "mouse-wheel"];

const REPLAY_CHECKPOINT_INTERVAL:number = 100003*1000*12;
/* Cycles per millisecond (appx) * milliseconds per second * number of seconds */

const REPLAY_VERSION=1;

const BLOCK_SIZE = 256;
const SUPERBLOCK_SIZE = 256;
const STATE_INDEX_INFO_LEN = 3;
const STATE_INFO_BLOCK_START = 16;

export enum ReplayMode {
  Inactive=0,
  Record,
  Playback,
  Finished,
}

export class Checkpoint {
  header_info:Uint8Array;
  superblock_seq:Uint32Array;
  name:string;
  thumbnail:string;
  when:number;
  event_index:number;
  new_blocks:number[];
  new_superblocks:number[];
  constructor(when:number, name:string, event_index:number, header_info:Uint8Array, new_blocks:number[], new_superblocks:number[], superblock_seq:Uint32Array, thumbnail:string) {
    this.when = when;
    this.name = name;
    this.event_index = event_index;
    this.header_info = header_info;
    this.superblock_seq = superblock_seq;
    this.new_blocks = new_blocks;
    this.new_superblocks = new_superblocks;
    this.thumbnail = thumbnail;
  }
}

export class Replay {
  events:ReplayEvent[]=[];
  checkpoints:Checkpoint[]=[];
  index:number=0;
  checkpoint_index:number=0;
  id:string="";
  mode:ReplayMode=ReplayMode.Inactive;
  last_time:number=0;
  wraps:number=0;
  superblock_index!: BlockIndex;
  block_index!: BlockIndex;
  version:number=REPLAY_VERSION;

  public static async create(id:string, mode:ReplayMode) : Promise<Replay> {
    const ths = new Replay();
    ths.id = id;
    ths.mode = mode;
    ths.block_index = await BlockIndex.create(BLOCK_SIZE); // measured in bytes
    ths.superblock_index = await BlockIndex.create(SUPERBLOCK_SIZE*4); // blocks*sizeof(int)
    return ths;
  }
  public async restore_checkpoint(header_info:Uint8Array, superblock_seq:Uint32Array):Promise<ArrayBuffer> {
    const block_byte_size = this.block_index.object_size;
    const superblock_byte_size = (this.superblock_index.object_size/4) * block_byte_size;
    /* TODO reuse this allocation across calls */
    const header_info_length = header_info.length;
    const state = new Uint8Array(header_info_length + superblock_seq.length * superblock_byte_size);
    state.set(header_info, 0);
    for (let i = 0; i < superblock_seq.length; i++) {
      const superblock_idx = superblock_seq[i];
      const superblock_bytes = this.superblock_index.get(superblock_idx);
      const superblock = new Uint32Array(superblock_bytes.buffer, superblock_bytes.byteOffset, superblock_bytes.byteLength / 4);
      const superblock_sz = superblock.length;
      for (let j = 0; j < superblock_sz; j++) {
        const block_idx = superblock[j];
        const byte_offset = header_info_length + i * superblock_byte_size + j * block_byte_size;
        state.set(this.block_index.get(block_idx), byte_offset);
      }
    }
    return state.buffer;
  }
  async reset_to_checkpoint(n:number, mode:ReplayMode, emulator:V86):Promise<Checkpoint[]> {
    const checkpoint = this.checkpoints[n];
    this.checkpoint_index = n+1;
    const state = await this.restore_checkpoint(checkpoint.header_info, checkpoint.superblock_seq);
    await emulator.restore_state(state);
    this.seek_internal(checkpoint.event_index, checkpoint.when);
    console.log("time check:",emulator.get_instruction_counter(),this.last_time,checkpoint.when);
    const dropped_checkpoints = mode == ReplayMode.Record ? this.checkpoints.slice(this.checkpoint_index) : [];
    this.resume(mode, emulator);
    console.log("time check B:",emulator.get_instruction_counter(),this.last_time,checkpoint.when);
    if (dropped_checkpoints.length) {
      this.block_index.remove_after(checkpoint.when);
      this.superblock_index.remove_after(checkpoint.when);
    }
    return dropped_checkpoints;
  }
  private seek_internal(event_index:number, t:number) {
    if(event_index > this.events.length) { throw "Seek: event index out of bounds"; }
    const [wraps, time] = this.cpu_time(t);
    if(event_index < this.events.length) {
      if(this.events[event_index].when < t) {
        console.log("evt time check",this.events[event_index].when,t);
        throw "Seek: current event is before given t";
      }
    }
    this.index = event_index;
    this.wraps = wraps;
    this.last_time = time;
  }
  private resume(mode:ReplayMode, emulator:V86) {
    // ensure emulator time is current time
    this.mode = mode;
    console.log("Resume",mode);
    emulator.v86.cpu.instruction_counter[0] = this.last_time;
    if (this.mode == ReplayMode.Record) {
      emulator.mouse_set_status(true);
      emulator.keyboard_set_status(true);
      // truncate if necessary
      this.events.length = this.index;
      this.checkpoints.length = this.checkpoint_index;
    } else if(this.mode == ReplayMode.Playback) {
      emulator.mouse_set_status(false);
      emulator.keyboard_set_status(false);
      // don't truncate!
    } else {
      throw "Resume: invalid mode";
    }
  }
  current_time():number {
    return this.replay_time(this.last_time);
  }
  replay_time(insn_counter:number) : number {
    let wrap_amt = 2**32-1;
    // how many full wraparounds we have done
    wrap_amt *= this.wraps;
    // add in the amount of leftover time, which we get from insn_counter
    wrap_amt += insn_counter;
    return wrap_amt;
  }
  cpu_time(t:number):[number,number] {
    const wraps = Math.floor(t / (2**32-1));
    const rem = t - (wraps * (2**32-1));
    return [wraps, rem];
  }
  log_evt(emulator:V86, code:Evt, val:object|number) {
    if(this.mode == ReplayMode.Record) {
      //console.log("R",this.replay_time(emulator.get_instruction_counter()), EvtNames[code], val);
      this.events.push(new ReplayEvent(this.replay_time(emulator.get_instruction_counter()), code, val));
      this.index += 1;
    }
  }
  async encode_checkpoint(time:number, event_index:number, screenshot:string, state:Uint8Array) : Promise<Checkpoint> {
    const header_block = new Int32Array(state.buffer, state.byteOffset, 4);
    const info_block_len = header_block[STATE_INDEX_INFO_LEN];
    const info_block_buffer = state.slice(0, STATE_INFO_BLOCK_START + info_block_len);
    state = state.subarray(16+info_block_buffer.length);
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
        if (block_start > state_size) {
          superblock_buf[j] = 0;
        } else {
          let result;
          if(block_start + block_byte_size > state_size) {
            block_buf.subarray(state_size - block_start).fill(0);
            block_buf.set(state.subarray(block_start, block_end));
            result = await this.block_index.insert(block_buf, time);
          } else {
            result = await this.block_index.insert(state.subarray(block_start, block_end), time);
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
    const checkpoint = new Checkpoint(time, "replay"+this.id+"-check"+this.checkpoints.length.toString(), event_index, info_block_buffer, new_blocks, new_superblocks, superblock_seq, screenshot);
    this.checkpoints.push(checkpoint);
    return checkpoint;
  }
  async make_checkpoint(emulator:V86) {
    // console.log("make cp",this.replay_time(emulator.get_instruction_counter()),this.index,this.checkpoints.length);
    const state = new Uint8Array(await emulator.save_state());
    const time = this.replay_time(emulator.get_instruction_counter());
    const screenshot = emulator.screen_make_screenshot();
    await this.encode_checkpoint(time, this.index, screenshot.src, state);
    this.checkpoint_index += 1;
  }
  async tick(emulator:V86) {
    const t = emulator.get_instruction_counter();
    if (t < this.last_time) { // counter wrapped around, increase wraps
      this.wraps += 1;
    }
    this.last_time = t;
    const real_t = this.replay_time(t);
    switch(this.mode) {
      case ReplayMode.Record: {
        let last_t = 0;
        if(this.checkpoints.length != 0) {
          last_t = this.checkpoints[this.checkpoints.length-1].when;
        }
        if(real_t - last_t > REPLAY_CHECKPOINT_INTERVAL) {
          await this.make_checkpoint(emulator);
        }
        break;
      }
      case ReplayMode.Playback: {
        // What is earlier: next checkpoint or next event?
        while(true) {
          const event_t = (this.index < this.events.length) ? this.events[this.index].when : Number.MAX_SAFE_INTEGER;
          const checkpoint_t = (this.checkpoint_index < this.checkpoints.length) ? this.checkpoints[this.checkpoint_index].when : Number.MAX_SAFE_INTEGER;
          const event_is_first = (event_t < checkpoint_t) && event_t < real_t;
          const checkpoint_is_first = (checkpoint_t <= event_t) && checkpoint_t < real_t;
          if(checkpoint_is_first) {
            const check = this.checkpoints[this.checkpoint_index];
            const state = await this.restore_checkpoint(check.header_info, check.superblock_seq);
            await emulator.restore_state(state);
            //console.log("Playback time check",this.replay_time(emulator.get_instruction_counter()),check.when);
            this.checkpoint_index += 1;
          } else if(event_is_first) {
            const evt = this.events[this.index];
            //console.log(real_t, EvtNames[evt.code], evt.value);
            emulator.bus.send(EvtNames[evt.code], evt.value);
            this.index += 1;
          } else {
            // neither checkpoint or event
            break;
          }
        }
        if(this.index < this.events.length) {
          // playback continues
        } else {
          // pause emu?
          this.finish_playback(emulator);
        }
        break;
      }
      case ReplayMode.Inactive:
      case ReplayMode.Finished:
        // do nothing
        break;
    }
  }
  static async start_recording(emulator:V86):Promise<Replay> {
    const r = await Replay.create(generateUUID(),ReplayMode.Record);
    emulator.v86.cpu.instruction_counter[0] = 0;
    r.make_checkpoint(emulator);
    return r;
  }
  private finish_playback(emulator:V86) {
    emulator.mouse_set_status(true);
    emulator.keyboard_set_status(true);
    this.mode = ReplayMode.Finished;
  }
  private async finish_recording(emulator:V86) {
    this.make_checkpoint(emulator);
    this.mode = ReplayMode.Finished;
  }
  async stop(emulator:V86) {
    if(this.mode == ReplayMode.Record) {
      await this.finish_recording(emulator);
    }
    if(this.mode == ReplayMode.Playback) {
      this.finish_playback(emulator);
    }
  }
  async start_playback(emulator:V86) {
    this.mode = ReplayMode.Playback;
    this.index = 0;
    this.checkpoint_index = 0;
    this.wraps = 0;
    this.last_time = 0;
    emulator.v86.cpu.instruction_counter[0] = 0;
    emulator.mouse_set_status(false);
    emulator.keyboard_set_status(false);
    // TODO read initial superblock seq?
  }
  /* TODO: this api should be in terms of a stream not a buffer, to account for very long/large replays; also maybe shouldn't serialize/deserialize the whole thing at once!! */
  async serialize():Promise<ArrayBuffer> {
    const frame_count = this.events.length;
    const checkpoint_count = this.checkpoints.length;
    const last_check = this.checkpoints[this.checkpoints.length-1];
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
    for(const evt of this.events) {
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
    for(const check of this.checkpoints) {
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
  /* TODO: this api should be in terms of a stream not a buffer, to account for very long/large replays */
  static async deserialize(buf: ArrayBuffer): Promise<Replay> {
    const events = [];
    const view = new DataView(buf);
    let x = 0;
    const magic = view.getUint32(x, true);
    x += 4;
    if (magic != 0x4C505256) {
      throw "Invalid magic, not a v86replay file";
    }
    // metadata: 16 bytes UUID; reserve the rest for later.
    const id = bytes_to_uuid(new Uint8Array(buf, x, 16));
    x += 16;
    // version
    const version = view.getUint8(x);
    x += 1;
    if (version > REPLAY_VERSION) {
      throw "Unrecognized replay version";
    }
    // empty metadata bytes
    x += 15;
    const frame_count = view.getUint32(x, true);
    x += 4;
    const checkpoint_count = view.getUint32(x, true);
    x += 4;
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
    const r = await Replay.create(id, ReplayMode.Inactive);
    r.version = version;
    r.events = events;
    for (let i = 0; i < checkpoint_count; i++) {
      const when_b = view.getBigUint64(x, true);
      x += 8;
      if (when_b > BigInt(Number.MAX_SAFE_INTEGER)) {
        console.log("cc", when_b);
        throw "When is too big";
      }
      const when = Number(when_b);
      if (BigInt(when) != when_b) {
        console.log(when, when_b);
        throw "When didn't match";
      }
      const event_index = view.getUint32(x, true);
      x += 4;
      const name = "check" + i.toString();
      const thumb_len = view.getUint32(x, true);
      x += 4;
      const thumb_bytes = new Uint8Array(thumb_len);
      thumb_bytes.set(new Uint8Array(buf, x, thumb_len));
      const thumb = await blobToDataURL(new Blob([thumb_bytes], { type: "image/png" }));
      x += thumb_len;
      if (version == 0) {
        const state_len = view.getUint32(x, true);
        x += 4;
        const src_buf = new Uint8Array(buf, x, state_len);
        const state = new Uint8Array(state_len);
        state.set(src_buf);
        x += state_len;
        /* reencode as next statestream state */
        await r.encode_checkpoint(when, event_index, thumb, state);
      } else {
        /* already state stream, read new blocks, new superblocks, superblock seq */
        const info_len = view.getUint32(x, true);
        x += 4;
        const info = new Uint8Array(buf, x, info_len);
        x += info_len;
        const new_blocks = [];
        const new_block_count = view.getUint32(x, true);
        x += 4;
        for (let i = 0; i < new_block_count; i++) {
          const block = new Uint8Array(buf, x, r.block_index.object_size);
          new_blocks.push(r.block_index.length());
          r.block_index.insert_exact(block.slice(), r.block_index.length(), when);
          x += r.block_index.object_size;
        }
        const new_superblocks = [];
        const new_superblock_count = view.getUint32(x, true);
        x += 4;
        for (let i = 0; i < new_superblock_count; i++) {
          const superblock = new Uint8Array(buf, x, r.superblock_index.object_size);
          new_superblocks.push(r.superblock_index.length());
          r.superblock_index.insert_exact(superblock.slice(), r.block_index.length(), when);
          x += r.superblock_index.object_size;
        }
        const superblock_seq_len = view.getUint32(x, true);
        x += 4;
        const superblock_seq = new Uint32Array(superblock_seq_len);
        for (let i = 0; i < superblock_seq_len; i++) {
          superblock_seq[i] = view.getUint32(x, true);
          x += 4;
        }
        r.checkpoints.push(new Checkpoint(when, name, event_index, info.slice(), new_blocks, new_superblocks, superblock_seq.slice(), thumb));
      }
    }
    r.index = 0;
    r.checkpoint_index = 0;
    r.last_time = 0;
    r.wraps = 0;
    return r;
  }
}
export class ReplayEvent {
  when:number;
  code:Evt;
  value:object|number;
  constructor(when:number, code:Evt, value:object|number) {
    this.when = when;
    this.code = code;
    this.value = value;
  }
}

function generateUUID():string { // Public Domain/MIT
  let d = new Date().getTime();//Timestamp
  let d2 = ((typeof performance !== 'undefined') && performance.now && (performance.now()*1000)) || 0;//Time in microseconds since page-load or 0 if unsupported
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    let r = Math.random() * 16;//random number between 0 and 16
    if(d > 0){//Use timestamp until depleted
      r = (d + r)%16 | 0;
      d = Math.floor(d/16);
    } else {//Use microseconds since page-load if supported
      r = (d2 + r)%16 | 0;
      d2 = Math.floor(d2/16);
    }
    return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
  });
}
function uuid_to_bytes(s:string):Uint8Array {
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
function bytes_to_uuid(buf:Uint8Array):string {
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

//https://stackoverflow.com/a/30407959
function dataURLToBlob(dataurl:string) {
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
function blobToDataURL(blob: Blob): Promise<string> {
  if (import.meta.vitest) {
    return Promise.resolve("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==");
  }
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error);
    reader.onabort = () => reject(new Error("Read aborted"));
    reader.readAsDataURL(blob);
  });
}

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;
  const IMG_STR = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
  it("encode", async () => {
    const replay = await Replay.create("123456789abcdef123456789abcdef", ReplayMode.Inactive);
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
    const cp1 = await replay.encode_checkpoint(0, 0, IMG_STR, fake_state);
    expect(cp1.header_info.length).toBe(20);
    expect(replay.block_index.length()).toBe(2);
    expect(replay.superblock_index.length()).toBe(2);
    expect(cp1.superblock_seq.length).toBe(2);
    expect(cp1.superblock_seq).toEqual(Uint32Array.from([0,1]));
    fake_state[fake_state.length-1] = 2;
    const cp2 = await replay.encode_checkpoint(1, 1, IMG_STR, fake_state);
    expect(replay.block_index.length()).toBe(3);
    expect(replay.superblock_index.length()).toBe(3);
    expect(cp2.superblock_seq.length).toBe(2);
    expect(cp2.superblock_seq).toEqual(Uint32Array.from([0,2]));
  });
  it("decode", async () => {
    const replay = await Replay.create("123456789abcdef123456789abcdef", ReplayMode.Inactive);
    const header = [
      0,0,0,0,
      0,0,0,0,
      0,0,0,0,
      4,0,0,0,
      1,2,3,4,
    ];
    const fake_state = new Uint8Array(16+256*256*2);
    fake_state.set(header);
    /* all 0s except the last byte */
    fake_state[fake_state.length-1] = 1;
    const cp1 = await replay.encode_checkpoint(0, 0, IMG_STR, fake_state);
    const state_at_cp1 = fake_state.slice().buffer;
    fake_state[fake_state.length-1] = 2;
    const cp2 = await replay.encode_checkpoint(1, 1, IMG_STR, fake_state);
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
    const replay = await Replay.create("123456789abcdef123456789abcdef", ReplayMode.Inactive);
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
    const cp1 = await replay.encode_checkpoint(0, 0, IMG_STR, fake_state);
    fake_state[fake_state.length-1] = 2;
    const cp2 = await replay.encode_checkpoint(1, 1, IMG_STR, fake_state);
    const ser = await replay.serialize();
    const replay2 = await Replay.deserialize(ser);
    expect(cp1.header_info).toEqual(replay2.checkpoints[0].header_info);
    expect(cp1.superblock_seq).toEqual(replay2.checkpoints[0].superblock_seq);
    expect(cp1.new_blocks).toEqual(replay2.checkpoints[0].new_blocks);
    expect(cp1.new_superblocks).toEqual(replay2.checkpoints[0].new_superblocks);
    expect(cp2.header_info).toEqual(replay2.checkpoints[1].header_info);
    expect(cp2.superblock_seq).toEqual(replay2.checkpoints[1].superblock_seq);
    expect(cp2.new_blocks).toEqual(replay2.checkpoints[1].new_blocks);
    expect(cp2.new_superblocks).toEqual(replay2.checkpoints[1].new_superblocks);
    expect(replay.block_index.get(0)).toEqual(replay2.block_index.get(0));
    expect(replay.block_index.get(1)).toEqual(replay2.block_index.get(1));
    expect(replay.block_index.get(2)).toEqual(replay2.block_index.get(2));
    expect(replay.superblock_index.get(0)).toEqual(replay2.superblock_index.get(0));
    expect(replay.superblock_index.get(1)).toEqual(replay2.superblock_index.get(1));
  });
}
