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
    let wraps = Math.floor(t / (2**32-1));
    let rem = t - (wraps * (2**32-1));
    return [wraps, rem];
  }
  log_evt(emulator:V86Starter, code:Evt, val:any) {
    if(this.mode == ReplayMode.Record) {
      //console.log("R",this.replay_time(emulator.get_instruction_counter()), EvtNames[code], val);
      this.events.push(new ReplayEvent(this.replay_time(emulator.get_instruction_counter()), code, val));
      this.index += 1;
    }
  }
  async make_checkpoint(emulator:V86Starter) {
    //console.log("make cp",this.replay_time(emulator.get_instruction_counter()),this.index);
    this.checkpoints.push(new Checkpoint(this.replay_time(emulator.get_instruction_counter()), "check"+this.checkpoints.length.toString(), this.index, await emulator.save_state(), emulator.screen_make_screenshot().src));
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
      case ReplayMode.Record:
        let last_t = 0;
        if(this.checkpoints.length != 0) {
          last_t = this.checkpoints[this.checkpoints.length-1].when;
        }
        if(real_t - last_t > REPLAY_CHECKPOINT_INTERVAL) {
          this.make_checkpoint(emulator);
        }
      break;
      case ReplayMode.Playback:
        // What is earlier: next checkpoint or next event?
        while(true) {
          let event_t = (this.index < this.events.length) ? this.events[this.index].when : Number.MAX_SAFE_INTEGER;
          let checkpoint_t = (this.checkpoint_index < this.checkpoints.length) ? this.checkpoints[this.checkpoint_index].when : Number.MAX_SAFE_INTEGER;
          let event_is_first = (event_t < checkpoint_t) && event_t < real_t;
          let checkpoint_is_first = (checkpoint_t <= event_t) && checkpoint_t < real_t;
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
  serialize():ArrayBuffer {
    // magic+metadata+frame count+checkpoint count+(frame count * max event size)+(checkpoint count * size of last checkpoint)
    const frame_count = this.events.length;
    const checkpoint_count = this.checkpoints.length;
    let last_check = this.checkpoints[this.checkpoints.length-1];
    let size_max = 4+32+4+4+(frame_count*(1+8+4*8))+(checkpoint_count*(8+4+4+4+last_check.state.byteLength+last_check.thumbnail.length));
    const ret = new ArrayBuffer(size_max);
    const view = new DataView(ret);
    // ASCII "VRPL" backwards ("LPRV") so it shows up as "VRPL" in binary
    let x = 0;
    view.setUint32(x,0x4C505256,true);
    x += 4;
    // metadata: 16 bytes UUID; reserve the rest for later.
    for(let i = 0; i < this.id.length; i++) {
      view.setUint8(x,this.id.charCodeAt(i));
      x += 1;
    }
    view.setUint32(x,frame_count,true);
    x += 4;
    view.setUint32(x,checkpoint_count,true);
    x += 4;
    for(let evt of this.events) {
      view.setUint8(x,evt.code);
      x += 1;
      view.setBigUint64(x,BigInt(evt.when),true);
      x += 8;
      switch(evt.code) {
      case Evt.KeyCode:
        view.setUint8(x,evt.value);
        x += 1;
        break;
      case Evt.MouseClick:
        view.setUint8(x,evt.value[0]);
        view.setUint8(x+1,evt.value[1]);
        view.setUint8(x+2,evt.value[2]);
        x += 3;
        break;
      case Evt.MouseDelta:
      case Evt.MouseWheel:
        view.setFloat64(x,evt.value[0],true);
        view.setFloat64(x+8,evt.value[1],true);
        x += 8*2;
        break;
      case Evt.MouseAbsolute:
        view.setFloat64(x,evt.value[0],true);
        view.setFloat64(x+8,evt.value[1],true);
        view.setFloat64(x+16,evt.value[2],true);
        view.setFloat64(x+24,evt.value[3],true);
        x += 8*4;
        break;
      default:
        throw "Unhandled event type";
      }
    }
    for(let check of this.checkpoints) {
      // the when
      view.setBigUint64(x,BigInt(check.when),true);
      x += 8;
      // the event index
      view.setUint32(x,check.event_index,true);
      x += 4;
      // the thumbnail; TODO decode base64
      view.setUint32(x,check.thumbnail.length,true);
      x += 4;
      for(let byte of check.thumbnail) {
        view.setUint8(x,byte.charCodeAt(0));
        x += 1;
      }
      // the state
      view.setUint32(x,check.state.byteLength,true);
      x += 4;
      let st_buf = new Uint8Array(check.state);
      let dst_buf = new Uint8Array(ret,x,st_buf.length);
      dst_buf.set(st_buf);
      x += st_buf.length;
    }
    return ret;
  }
  static deserialize(buf:ArrayBuffer):Replay {
    let events = [];
    let checkpoints = [];
    const view = new DataView(buf);
    let x = 0;
    let magic = view.getUint32(x,true);
    x += 4;
    if(magic != 0x4C505256) {
      throw "Invalid magic, not a v86replay file";
    }
    // metadata: 16 bytes UUID; reserve the rest for later.
    let id = "";
    for(let i = 0; i < 16; i++) {
      id += String.fromCharCode(view.getUint8(x));
      x += 1;
    }
    // empty metadata bytes
    x += 16;
    let frame_count = view.getUint32(x,true);
    x += 16;
    let checkpoint_count = view.getUint32(x,true);
    x += 16;
    for(let i = 0; i < frame_count; i++) {
      let code = view.getUint8(x);
      x += 1;
      let when_b = view.getBigUint64(x,true);
      x += 8;
      if(when_b > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw "When is too big";
      }
      let when = Number(when_b);
      if(BigInt(when) != when_b) {
        throw "When didn't match";
      }
      let value:any;
      switch(code) {
      case Evt.KeyCode:
        value = view.getUint8(x);
        x += 1;
        break;
      case Evt.MouseClick: {
        let a = view.getUint8(x);
        let b = view.getUint8(x+1);
        let c = view.getUint8(x+2);
        x += 3;
        value = [a,b,c];
        break;
      }
      case Evt.MouseDelta:
      case Evt.MouseWheel: {
        let mx = view.getFloat64(x,true);
        let my = view.getFloat64(x+8,true);
        value = [mx,my];
        x += 8*2;
        break;
      }
      case Evt.MouseAbsolute: {
        let mx = view.getFloat64(x,true);
        let my = view.getFloat64(x+8,true);
        let w = view.getFloat64(x+16,true);
        let h = view.getFloat64(x+24,true);
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
      let when_b = view.getBigUint64(x,true);
      x += 8;
      if(when_b > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw "When is too big";
      }
      let when = Number(when_b);
      if(BigInt(when) != when_b) {
        throw "When didn't match";
      }
      let event_index = view.getUint32(x,true);
      x += 4;
      let name = "check"+i.toString();
      let thumb_len = view.getUint32(x,true);
      x += 4;
      // TODO: get binary and encdoe to base64
      let thumb = "";
      for(let j = 0; j < thumb_len; j++) {
        thumb += String.fromCharCode(view.getUint8(x+j));
      }
      x += thumb_len;
      let state_len = view.getUint32(x,true);
      x += 4;
      let src_buf = new Uint8Array(buf,x,state_len);
      let state = new Uint8Array(state_len);
      state.set(src_buf);
      x += state_len;
      checkpoints.push(new Checkpoint(when, name, event_index, state, thumb));
    }
    let r = new Replay(id, ReplayMode.Inactive);
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
  value:any;
  constructor(when:number, code:Evt, value:any) {
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
