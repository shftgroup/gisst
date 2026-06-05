import {BlockIndex} from './blockindex';
import {memcmp} from './utils';

export enum Evt {
  KeyCode = 0,
  MouseClick = 1,
  MouseDelta = 2,
  MouseAbsolute = 3,
  MouseWheel = 4
}
export const EvtNames:string[] = ["keyboard-code", "mouse-click", "mouse-delta", "mouse-absolute", "mouse-wheel"];

//const REPLAY_CHECKPOINT_INTERVAL:number = 100003*1000*4;
/* Cycles per millisecond (appx) * milliseconds per second * number of seconds */

const REPLAY_VERSION=1;

const BLOCK_SIZE = 256;
const SUPERBLOCK_SIZE = 256;
const STATE_INDEX_INFO_LEN = 3;
const STATE_INFO_BLOCK_START = 16;

export enum ReplayMode {
  Inactive=0,
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

export class LegacyReplay {
  events:ReplayEvent[]=[]; // replace with file
  checkpoints:Checkpoint[]=[]; // replace with file
  index:number=0;
  checkpoint_index:number=0;
  id:string="";
  mode:ReplayMode=ReplayMode.Inactive;
  last_time:number=0;
  wraps:number=0; // store in file??
  superblock_index!: BlockIndex; // read from / write to file
  block_index!: BlockIndex; // read from / write to file
  version:number=REPLAY_VERSION; // store in file header
  last_state?:Uint8Array;
  public static async create(id:string, mode:ReplayMode) : Promise<LegacyReplay> {
    const ths = new LegacyReplay();
    ths.id = id;
    ths.mode = mode;
    ths.block_index = await BlockIndex.create(BLOCK_SIZE); // measured in bytes
    ths.superblock_index = await BlockIndex.create(SUPERBLOCK_SIZE*4); // blocks*sizeof(int)
    // create file
    return ths;
  }

  public async restore_checkpoint(header_info:Uint8Array, superblock_seq:Uint32Array):Promise<ArrayBuffer> {
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
    this.last_state = state.slice(header_info_length);
    return buffer;
  }
  async reset_to_checkpoint(_n:number, mode:ReplayMode, emulator:V86):Promise<Checkpoint[]> {
    // for file: rewind or fast forward until checkpoint is found, can't just skip like this
    // note that old checkpoints might not be encoded correctly so we have to ignore the input checkpoint number n
    const checkpoint = this.checkpoints[0];
    this.checkpoint_index = 1;
    const state = await this.restore_checkpoint(checkpoint.header_info, checkpoint.superblock_seq);
    await emulator.restore_state(state);
    this.seek_internal(checkpoint.event_index, checkpoint.when);
    this.resume(mode, emulator);
    return [];
  }
  private seek_internal(event_index:number, t:number) {
    // seek file, do reads
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
    if(this.mode == ReplayMode.Playback) {
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
  async encode_checkpoint(time:number, event_index:number, screenshot:string, state:Uint8Array) : Promise<Checkpoint> {
    // write to file
    const header_block = new Int32Array(state.buffer, state.byteOffset, 4);
    const info_block_len = header_block[STATE_INDEX_INFO_LEN];
    // TODO: maybe use jsondiff / microdiff to take a diff of this info block from the last one
    const info_block_buffer = state.slice(0, STATE_INFO_BLOCK_START + info_block_len);
    const orig_state = state;
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
    const USE_MEMCMP = false;
    for (let i = 0; i < superblock_count; i++) {
      const superblock_offset = i * superblock_byte_size;
      for (let j = 0; j < superblock_block_count; j++) {
        const block_start = superblock_offset + j * block_byte_size;
        const block_end = Math.min(block_start + block_byte_size, state_size);
        if (block_start >= state_size) {
          superblock_buf[j] = 0;
        } else if (USE_MEMCMP && this.last_state && memcmp(this.last_state.subarray(block_start, block_end), state.subarray(block_start, block_end))) {
          const old_super = this.checkpoints[this.checkpoints.length-1].superblock_seq[i];
          const old_super_contents = this.superblock_index.get(old_super);
          superblock_buf[j] = old_super_contents[j];
          continue;
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
    const checkpoint = new Checkpoint(time, "replay"+this.id+"-state"+this.checkpoints.length.toString(), event_index, info_block_buffer, new_blocks, new_superblocks, superblock_seq, screenshot);
    this.checkpoints.push(checkpoint);
    if (USE_MEMCMP) {
      this.last_state = state;
    }
    const DO_ROUNDTRIP = false;
    if (DO_ROUNDTRIP) {
      const restore = await this.restore_checkpoint(checkpoint.header_info, checkpoint.superblock_seq);
      const roundtrip = new Uint8Array(restore);
      if (roundtrip.length != orig_state.length) {
        console.error("Bad encode/decode len",orig_state.length,roundtrip.length);
        throw "ohno";
      }
      for (let i = 0; i < roundtrip.length; i++) {
        if (orig_state[i] != roundtrip[i]) {
          console.error("encode differs from decode at",i,":",orig_state[i]," vs ",roundtrip[i]);
          throw "uhoh";
        }
      }
    }
    return checkpoint;
  }
  async tick(emulator:V86) {
    // read from / write to file
    const t = emulator.get_instruction_counter();
    if (t < this.last_time) { // counter wrapped around, increase wraps
      this.wraps += 1;
    }
    this.last_time = t;
    const real_t = this.replay_time(t);
    switch(this.mode) {
      case ReplayMode.Playback: {
        emulator.keyboard_set_status(false);
        emulator.mouse_set_status(false);
        // What is earlier: next checkpoint or next event?
        while(true) {
          const event_t = (this.index < this.events.length) ? this.events[this.index].when : Number.MAX_SAFE_INTEGER;
          const event_coming = event_t <= real_t;
          if(event_coming) {
            const evt = this.events[this.index];
            emulator.bus.send(EvtNames[evt.code], evt.value);
            this.index += 1;
          } else {
            // out of events
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
  private finish_playback(emulator:V86) {
    emulator.mouse_set_status(true);
    emulator.keyboard_set_status(true);
    this.mode = ReplayMode.Finished;
  }
  async stop(emulator:V86) {
    if(this.mode == ReplayMode.Playback) {
      this.finish_playback(emulator);
    }
  }
  async start_playback(emulator:V86) {
    this.mode = ReplayMode.Playback;
    this.index = 0;
    this.wraps = 0;
    this.last_time = 0;
    emulator.mouse_set_status(false);
    emulator.keyboard_set_status(false);
    const check = this.checkpoints[0];
    const state = await this.restore_checkpoint(check.header_info, check.superblock_seq);
    await emulator.restore_state(state);
  }
  /* TODO: this api should be in terms of a stream not a buffer, to account for very long/large replays */
  // will just need to read header from file
  static async deserialize(buf: ArrayBuffer): Promise<LegacyReplay> {
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
    // these will be read as we go
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
    const r = await LegacyReplay.create(id, ReplayMode.Inactive);
    r.version = version;
    r.events = events;
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
      } else if (version == 1) {
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
    r.last_state = undefined;
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

//https://stackoverflow.com/a/67551175
function blobToDataURL(blob: Blob): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error);
    reader.onabort = () => reject(new Error("Read aborted"));
    reader.readAsDataURL(blob);
  });
}
