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
