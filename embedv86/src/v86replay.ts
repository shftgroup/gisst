export enum Evt {
  Checkpoint = 0,
  KeyCode = 1,
  MouseClick = 2,
  MouseDelta = 3,
  MouseAbsolute = 4,
  MouseWheel = 5
}
export const EvtNames:(string|null)[] = [null, "keyboard-code", "mouse-click", "mouse-delta", "mouse-absolute", "mouse-wheel"];

const REPLAY_CHECKPOINT_INTERVAL:bigint = BigInt(10003*1000*120);
/* Cycles per millisecond (appx) * milliseconds per second * number of seconds */

enum ReplayMode {
  Inactive=0,
  Record,
  Playback,
  Finished,
}

export class Replay {
  events:ReplayEvent[];
  index:number;
  id:string;
  mode:ReplayMode;
  last_time:number;
  wraps:number;
  
  private constructor(id:string, mode:ReplayMode) {
    this.id = id;
    this.events = [];
    this.index = 0;
    this.wraps = 0;
    this.last_time = 0;
    this.mode = mode;
  }
  replay_time(insn_counter:number) : bigint {
    let wrap_amt = BigInt(2**32-1);
    // how many full wraparounds we have done
    wrap_amt *= BigInt(this.wraps);
    // add in the amount of leftover time, which we get from insn_counter
    wrap_amt += BigInt(insn_counter);
    return wrap_amt;
  }
  log_evt(emulator:V86Starter, code:Evt, val:any) {
    if(this.mode == ReplayMode.Record) {
      this.events.push(new ReplayEvent(this.replay_time(emulator.get_instruction_counter()), code, val));
      console.log(EvtNames[code],this.events[this.events.length-1]);
    }
  }
  async tick(emulator:V86Starter) {
    const t = emulator.get_instruction_counter();
    if (t < this.last_time) { // counter wrapped around, increase wraps
      this.wraps += 1;
    }
    const real_t = this.replay_time(t);
    switch(this.mode) {
      case ReplayMode.Record:
        let last_t = BigInt(0);
        if(this.events.length != 0) {
          last_t = this.events[this.events.length-1].when;
        }
        if(real_t - last_t > REPLAY_CHECKPOINT_INTERVAL) {
          this.events.push(new ReplayEvent(real_t, Evt.Checkpoint, await emulator.save_state()));
        }
      break;
      case ReplayMode.Playback:
        if(this.index < this.events.length) {
          let next_t = this.events[this.index].when;
          while(next_t <= t) {
            const evt = this.events[this.index];
            if(evt.code == Evt.Checkpoint) {
              emulator.restore_state(evt.value);
            } else {
              emulator.bus.send(EvtNames[evt.code], evt.value);
            }
            this.index += 1;
            if(this.index < this.events.length) {
              next_t = this.events[this.index].when;
            } else {
              break;
            }
          }
          if(this.index < this.events.length) {
            // playback continues
          } else {
            this.finish_playback(emulator);
            break;
          }
        }
        else {
          // pause emu?
          this.finish_playback(emulator);
          break;
        }
        break;
      case ReplayMode.Inactive:
      case ReplayMode.Finished:
        // do nothing
        break;
    }
    this.last_time = t;
  }
  static async start_recording(emulator:V86Starter):Promise<Replay> {
    const r = new Replay(generateUUID(),ReplayMode.Record);
    emulator.v86.cpu.instruction_counter[0] = 0;
    r.log_evt(emulator,Evt.Checkpoint,await emulator.save_state());
    return r;
  }
  private finish_playback(emulator:V86Starter) {
    emulator.mouse_set_status(true);
    emulator.keyboard_set_status(true);
    this.mode = ReplayMode.Finished;
  }
  private async finish_recording(emulator:V86Starter) {
    this.log_evt(emulator,Evt.Checkpoint,await emulator.save_state());
    this.mode = ReplayMode.Finished;
  }
  async stop(emulator:V86Starter) {
    if(this.mode == ReplayMode.Record) {
      await this.finish_recording(emulator);
    }
    if(this.mode == ReplayMode.Playback) {
      this.finish_playback(emulator);
    }
    console.log(this);
  }
  async start_playback(emulator:V86Starter) {
    this.mode = ReplayMode.Playback;
    this.index = 0;
    emulator.v86.cpu.instruction_counter[0] = 0;
    emulator.mouse_set_status(false);
    emulator.keyboard_set_status(false);
    this.wraps = 0;
    this.last_time = 0;
  }
}
class ReplayEvent {
  when:bigint;
  code:Evt;
  value:any;
  constructor(when:bigint, code:Evt, value:any) {
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
