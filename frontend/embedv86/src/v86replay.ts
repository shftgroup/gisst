export enum Evt {
  KeyCode = 0,
  MouseClick = 1,
  MouseDelta = 2,
  MouseAbsolute = 3,
  MouseWheel = 4
}
export const EvtNames:string[] = ["keyboard-code", "mouse-click", "mouse-delta", "mouse-absolute", "mouse-wheel"];

const REPLAY_CHECKPOINT_INTERVAL:number = 10003*1000*120;
/* Cycles per millisecond (appx) * milliseconds per second * number of seconds */

export enum ReplayMode {
  Inactive=0,
  Record,
  Playback,
  Finished,
}

export class Checkpoint {
  state:ArrayBuffer;
  name:string;
  thumbnail:string;
  when:number;
  event_index:number;
  constructor(when:number, name:string, event_index:number, state:ArrayBuffer, thumbnail:string) {
    this.when = when;
    this.name = name;
    this.event_index = event_index;
    this.state = state;
    this.thumbnail = thumbnail;
  }
}

export class Replay {
  events:ReplayEvent[];
  checkpoints:Checkpoint[];
  index:number;
  checkpoint_index:number;
  id:string;
  mode:ReplayMode;
  last_time:number;
  wraps:number;
  
  private constructor(id:string, mode:ReplayMode) {
    this.id = id;
    this.events = [];
    this.checkpoints = [];
    this.index = 0;
    this.checkpoint_index = 0;
    this.wraps = 0;
    this.last_time = 0;
    this.mode = mode;
  }
  reset_to_checkpoint(n:number, mode:ReplayMode, emulator:V86Starter):Checkpoint[] {
    const checkpoint = this.checkpoints[n];
    this.checkpoint_index = n+1;
    emulator.restore_state(checkpoint.state);
    this.seek_internal(checkpoint.event_index, checkpoint.when);
    console.log("time check:",emulator.get_instruction_counter(),this.last_time,checkpoint.when);
    const dropped_checkpoints = mode == ReplayMode.Record ? this.checkpoints.slice(this.checkpoint_index) : [];
    this.resume(mode, emulator);
    console.log("time check B:",emulator.get_instruction_counter(),this.last_time,checkpoint.when);
    return dropped_checkpoints;
  }
  // TODO: add a seek api that respects checkpoints
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
  private resume(mode:ReplayMode, emulator:V86Starter) {
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
  log_evt(emulator:V86Starter, code:Evt, val:object|number) {
    if(this.mode == ReplayMode.Record) {
      //console.log("R",this.replay_time(emulator.get_instruction_counter()), EvtNames[code], val);
      this.events.push(new ReplayEvent(this.replay_time(emulator.get_instruction_counter()), code, val));
      this.index += 1;
    }
  }
  async make_checkpoint(emulator:V86Starter) {
    // console.log("make cp",this.replay_time(emulator.get_instruction_counter()),this.index,this.checkpoints.length);
    this.checkpoints.push(new Checkpoint(this.replay_time(emulator.get_instruction_counter()), "replay"+this.id+"-check"+this.checkpoints.length.toString(), this.index, await emulator.save_state(), emulator.screen_make_screenshot().src));
    this.checkpoint_index += 1;
  }
  async tick(emulator:V86Starter) {
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
          this.make_checkpoint(emulator);
        }
        break;
      }
      case ReplayMode.Playback: {
        // What is earlier: next checkpoint or next event?
        /* eslint-disable no-constant-condition */
        while(true) {
          const event_t = (this.index < this.events.length) ? this.events[this.index].when : Number.MAX_SAFE_INTEGER;
          const checkpoint_t = (this.checkpoint_index < this.checkpoints.length) ? this.checkpoints[this.checkpoint_index].when : Number.MAX_SAFE_INTEGER;
          const event_is_first = (event_t < checkpoint_t) && event_t < real_t;
          const checkpoint_is_first = (checkpoint_t <= event_t) && checkpoint_t < real_t;
          if(checkpoint_is_first) {
            const check = this.checkpoints[this.checkpoint_index];
            emulator.restore_state(check.state);
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
  static async start_recording(emulator:V86Starter):Promise<Replay> {
    const r = new Replay(generateUUID(),ReplayMode.Record);
    emulator.v86.cpu.instruction_counter[0] = 0;
    r.make_checkpoint(emulator);
    return r;
  }
  private finish_playback(emulator:V86Starter) {
    emulator.mouse_set_status(true);
    emulator.keyboard_set_status(true);
    this.mode = ReplayMode.Finished;
  }
  private async finish_recording(emulator:V86Starter) {
    this.make_checkpoint(emulator);
    this.mode = ReplayMode.Finished;
  }
  async stop(emulator:V86Starter) {
    if(this.mode == ReplayMode.Record) {
      await this.finish_recording(emulator);
    }
    if(this.mode == ReplayMode.Playback) {
      this.finish_playback(emulator);
    }
    // console.log(this);
  }
  async start_playback(emulator:V86Starter) {
    this.mode = ReplayMode.Playback;
    this.index = 0;
    this.checkpoint_index = 0;
    this.wraps = 0;
    this.last_time = 0;
    emulator.v86.cpu.instruction_counter[0] = 0;
    emulator.mouse_set_status(false);
    emulator.keyboard_set_status(false);
  }
  async serialize():Promise<ArrayBuffer> {
    // magic+metadata+frame count+checkpoint count+(frame count * max event size)+(checkpoint count * size of last checkpoint)
    const frame_count = this.events.length;
    const checkpoint_count = this.checkpoints.length;
    const last_check = this.checkpoints[this.checkpoints.length-1];
    const size_max = 4+32+4+4+(frame_count*(1+8+4*8))+(checkpoint_count*(8+4+4+4+last_check.state.byteLength+last_check.thumbnail.length));
    const ret = new ArrayBuffer(size_max);
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
    // empty 16 bytes
    x += 16;
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
      switch(evt.code) {
      case Evt.KeyCode:
        view.setUint8(x,evt.value as number);
        x += 1;
        break;
      case Evt.MouseClick:
        view.setUint8(x,(evt.value as number[])[0]);
        view.setUint8(x+1,(evt.value as number[])[1]);
        view.setUint8(x+2,(evt.value as number[])[2]);
        x += 3;
        break;
      case Evt.MouseDelta:
      case Evt.MouseWheel:
        view.setFloat64(x,evt.value as number[][0],true);
        view.setFloat64(x+8,evt.value as number[][1],true);
        x += 8*2;
        break;
      case Evt.MouseAbsolute:
        view.setFloat64(x,evt.value as number[][0],true);
        view.setFloat64(x+8,evt.value as number[][1],true);
        view.setFloat64(x+16,evt.value as number[][2],true);
        view.setFloat64(x+24,evt.value as number[][3],true);
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
        const dst = new Uint8Array(ret,x,thumb_bytes.byteLength);
        dst.set(new Uint8Array(thumb_bytes));
      }
      x += thumb_bytes.byteLength;
      // the state
      view.setUint32(x,check.state.byteLength,true);
      x += 4;
      const st_buf = new Uint8Array(check.state);
      const dst_buf = new Uint8Array(ret,x,check.state.byteLength);
      dst_buf.set(st_buf);
      x += check.state.byteLength;
    }
    return ret;
  }
  static async deserialize(buf:ArrayBuffer):Promise<Replay> {
    const events = [];
    const checkpoints = [];
    const view = new DataView(buf);
    let x = 0;
    const magic = view.getUint32(x,true);
    x += 4;
    if(magic != 0x4C505256) {
      throw "Invalid magic, not a v86replay file";
    }
    // metadata: 16 bytes UUID; reserve the rest for later.
    const id = bytes_to_uuid(new Uint8Array(buf,x,16));
    x += 16;
    // empty metadata bytes
    x += 16;
    const frame_count = view.getUint32(x,true);
    x += 4;
    const checkpoint_count = view.getUint32(x,true);
    x += 4;
    for(let i = 0; i < frame_count; i++) {
      const code = view.getUint8(x);
      x += 1;
      const when_b = view.getBigUint64(x,true);
      x += 8;
      if(when_b > BigInt(Number.MAX_SAFE_INTEGER)) {
        console.log(when_b);
        throw "When is too big";
      }
      const when = Number(when_b);
      if(BigInt(when) != when_b) {
        console.log(when,when_b);
        throw "When didn't match";
      }
      let value:object|number;
      switch(code) {
      case Evt.KeyCode:
        value = view.getUint8(x);
        x += 1;
        break;
      case Evt.MouseClick: {
        const a = view.getUint8(x);
        const b = view.getUint8(x+1);
        const c = view.getUint8(x+2);
        x += 3;
        value = [a,b,c];
        break;
      }
      case Evt.MouseDelta:
      case Evt.MouseWheel: {
        const mx = view.getFloat64(x,true);
        const my = view.getFloat64(x+8,true);
        value = [mx,my];
        x += 8*2;
        break;
      }
      case Evt.MouseAbsolute: {
        const mx = view.getFloat64(x,true);
        const my = view.getFloat64(x+8,true);
        const w = view.getFloat64(x+16,true);
        const h = view.getFloat64(x+24,true);
        value = [mx,my,w,h];
        x += 8*4;
        break;
      }
      default:
        throw "Unhandled event type";
      }
      events.push(new ReplayEvent(when, code, value));
    }
    for(let i = 0; i < checkpoint_count; i++) {
      const when_b = view.getBigUint64(x,true);
      x += 8;
      if(when_b > BigInt(Number.MAX_SAFE_INTEGER)) {
        console.log(when_b);
        throw "When is too big";
      }
      const when = Number(when_b);
      if(BigInt(when) != when_b) {
        console.log(when,when_b);
        throw "When didn't match";
      }
      const event_index = view.getUint32(x,true);
      x += 4;
      const name = "check"+i.toString();
      const thumb_len = view.getUint32(x,true);
      x += 4;
      const thumb_bytes = new Uint8Array(thumb_len);
      thumb_bytes.set(new Uint8Array(buf,x,thumb_len));
      const thumb = await blobToDataURL(new Blob([thumb_bytes], {type:"image/png"}));
      x += thumb_len;
      const state_len = view.getUint32(x,true);
      x += 4;
      const src_buf = new Uint8Array(buf,x,state_len);
      const state = new Uint8Array(state_len);
      state.set(src_buf);
      x += state_len;
      checkpoints.push(new Checkpoint(when, name, event_index, state, thumb));
    }
    const r = new Replay(id, ReplayMode.Inactive);
    r.events = events;
    r.checkpoints = checkpoints;
    r.index = 0;
    r.checkpoint_index = 0;
    r.last_time = 0;
    r.wraps = 0;
    return r;
  }
}
class ReplayEvent {
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
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error);
    reader.onabort = () => reject(new Error("Read aborted"));
    reader.readAsDataURL(blob);
  });
}
