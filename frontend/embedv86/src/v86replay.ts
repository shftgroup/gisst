import {LegacyReplay} from './v86legacyreplay';
import {nonnull,bytes_to_uuid} from './utils';
import {WorkerResponse} from './worker_protocol.d';
import ReplayWorker from './replay_worker.ts?worker&inline';

export enum Evt {
  KeyCode = 0,
  MouseClick = 1,
  MouseDelta = 2,
  MouseAbsolute = 3,
  MouseWheel = 4
};
export const EvtNames:string[] = ["keyboard-code", "mouse-click", "mouse-delta", "mouse-absolute", "mouse-wheel"];
const REPLAY_CHECKPOINT_INTERVAL:number = 100003*1000*12;
/* Cycles per millisecond (appx) * milliseconds per second * number of seconds */

const REPLAY_VERSION = 1;
const REPLAY_LEGACY_VERSION = 0;

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
  events:ReplayEvent[]=[]; // replace with file
  checkpoints:Checkpoint[]=[]; // replace with file
  index:number=0;
  checkpoint_index:number=0;
  id:string="";
  mode:ReplayMode=ReplayMode.Inactive;
  last_time:number=0;
  worker!:Worker;
  wraps:number=0;
  version:number=REPLAY_VERSION;
  pending_serialize:((ab:ArrayBuffer)=>void)|null = null;
  pending_deserialize:((data:{id:string,version:number,events:ReplayEvent[],checkpoints:Checkpoint[]}) => void) | null = null;
  pending_decode_number:number = 0;
  pending_decode:((ab:ArrayBuffer)=>void)|null = null;
  video:File|null = null;
  recording_video:Blob[] = [];
  recorder: MediaRecorder|null = null;
  container: HTMLDivElement|null = null;

  static async create(id:string, mode:ReplayMode):Promise<Replay>{
    const ths = new Replay();
    ths.id = id;
    ths.mode = mode;
    ths.worker = new ReplayWorker();
    return new Promise((resolve,_reject) => {
      ths.worker.addEventListener("message", (msg) => {
      const data = msg.data as WorkerResponse;
        switch (data.type) {
          case "initialized": {
            resolve(ths);
            break;
          }
          case "checkpoint": {
            const cp = ths.checkpoints[data.args.which];
            cp.header_info = data.args.header_info;
            cp.superblock_seq = data.args.superblock_seq;
            cp.new_blocks = data.args.new_blocks;
            cp.new_superblocks = data.args.new_superblocks;
            break;
          }
          case "serialize":{
            nonnull(ths.pending_serialize);
            ths.pending_serialize(data.args.result);
            ths.pending_serialize = null;
            break;
          }
          case "deserialize": {
            nonnull(ths.pending_deserialize);
            ths.pending_deserialize(data.args);
            ths.pending_deserialize = null;
            break;
          }
          case "decode_checkpoint": {
            if (ths.pending_decode_number == data.args.which) {
              nonnull(ths.pending_decode);
              ths.pending_decode(data.args.result);
              ths.pending_decode = null;
            }
            break;
          }
          default:
            break;
        }
      });
      ths.worker.postMessage({type:"init", args:{id:ths.id,version:ths.version}});
    });
  }
  public async restore_checkpoint(header_info:Uint8Array, superblock_seq:Uint32Array):Promise<ArrayBuffer> {
    return new Promise((resolve,_reject) => {
      if (this.pending_decode) {
        console.warn("Checkpoint restore asked for while in progress; original restore will be ignored");
      }
      this.pending_decode_number++;
      const which = this.pending_decode_number;
      this.pending_decode = resolve;
      this.worker.postMessage({type:"decode", args:{which, header_info, superblock_seq}});
    });
  }
  async reset_to_checkpoint(n:number, mode:ReplayMode, emulator:V86):Promise<Checkpoint[]> {
    if (mode == ReplayMode.Record) {
      throw "Can't reset to checkpoint during recording for now";
    }
    // for file: rewind or fast forward until checkpoint is found, can't just skip like this
    const checkpoint = this.checkpoints[n];
    const state = await this.restore_checkpoint(checkpoint.header_info, checkpoint.superblock_seq);
    await emulator.restore_state(state);
    this.seek_internal(n+1, checkpoint.event_index, checkpoint.when);
    //    const dropped_checkpoints = mode == ReplayMode.Record ? this.checkpoints.slice(this.checkpoint_index) : [];
    const dropped_checkpoints:Checkpoint[] = [];
    this.resume(mode, emulator);
    if (dropped_checkpoints.length) {
      this.worker.postMessage({type:"drop_checkpoints", args:{when:checkpoint.when}});
    }
    return dropped_checkpoints;
  }
  private seek_internal(checkpoint_index:number, event_index:number, t:number) {
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
    this.checkpoint_index = checkpoint_index;
    if (this.video) {
      const end_time = Math.max(this.events[this.events.length-1].when, this.checkpoints[this.checkpoints.length-1].when);
      const ratio = t / end_time;
      const video_elt = this.container!.querySelector("video")!;
      video_elt.currentTime = ratio * video_elt.duration;
    }
    this.wraps = wraps;
    this.last_time = time;
  }
  private resume(mode:ReplayMode, emulator:V86) {
    // ensure emulator time is current time
    this.mode = mode;
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
    if (this.video && this.mode == ReplayMode.Playback) {
      const video_elt = this.container!.querySelector("video")!;
      const replay_duration = Math.max(this.events[this.events.length-1].when, this.checkpoints[this.checkpoints.length-1].when);
      return (video_elt.currentTime / video_elt.duration) * replay_duration;
    } else {
      return this.replay_time(this.last_time);
    }
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
    // write to file
    if(this.mode == ReplayMode.Record) {
      this.events.push(new ReplayEvent(this.replay_time(emulator.get_instruction_counter()), code, val));
      this.index += 1;
    }
  }
  // TODO: Make it return a promise that resolves when the checkpoint is encoded 
  public make_checkpoint(emulator:V86, state:Uint8Array) {
    const time = this.current_time();
    const screenshot = emulator.screen_make_screenshot();
    const checkpoint = new Checkpoint(time, "replay"+this.id+"-state"+this.checkpoints.length.toString(), this.index, new Uint8Array(), [], [], new Uint32Array(), screenshot.src);
    this.checkpoints.push(checkpoint);
    this.worker.postMessage({type:"encode", args:{time,state:state, which:this.checkpoint_index}}, {transfer:[state.buffer]});
    this.checkpoint_index += 1;
  }
  async tick(emulator:V86) {
    // read from / write to file
    if (!(this.mode == ReplayMode.Playback && this.video)) {
      const t = emulator.v86.cpu.instruction_counter[0];
      if (t < this.last_time) { // counter wrapped around, increase wraps
        this.wraps += 1;
      }
      this.last_time = t;
    }
    const real_t = this.current_time();
    switch(this.mode) {
      case ReplayMode.Record: {
        let last_t = 0;
        if (this.recorder) {
          const canvas = this.container!.getElementsByTagName("canvas")[0]!;
          if (canvas.style.display == "none") {
            // hidden because we're in text mode, so we absolutely must make a screenshot to get that into the mediastream!
            const screen = emulator.screen_make_screenshot();
            canvas.width = screen.width;
            canvas.height = screen.height;
            canvas.getContext("2d")!.drawImage(screen, 0, 0);
          }
        }
        if(this.checkpoints.length != 0) {
          last_t = this.checkpoints[this.checkpoints.length-1].when;
        }
        if(real_t - last_t > REPLAY_CHECKPOINT_INTERVAL) {
          this.make_checkpoint(emulator,new Uint8Array(await emulator.save_state()));
        }
        break;
      }
      case ReplayMode.Playback: {
        // What is earlier: next checkpoint or next event?
        emulator.keyboard_set_status(false);
        emulator.mouse_set_status(false);
        while(true) {
          const event_t = (this.index < this.events.length) ? this.events[this.index].when : Number.MAX_SAFE_INTEGER;
          const checkpoint_t = (this.checkpoint_index < this.checkpoints.length) ? this.checkpoints[this.checkpoint_index].when : Number.MAX_SAFE_INTEGER;
          const event_is_first = (event_t < checkpoint_t) && event_t <= real_t;
          const checkpoint_is_first = (checkpoint_t <= event_t) && checkpoint_t <= real_t;
          if(checkpoint_is_first) {
            const check = this.checkpoints[this.checkpoint_index];
            this.checkpoint_index += 1;
            const state = await this.restore_checkpoint(check.header_info, check.superblock_seq);
            await emulator.restore_state(state);
          } else if(event_is_first) {
            const evt = this.events[this.index];
            if (!this.video) {
              emulator.bus.send(EvtNames[evt.code], evt.value);
            }
            this.index += 1;
          } else {
            // neither checkpoint or event
            break;
          }
        }
        if(this.index < this.events.length || this.checkpoint_index < this.checkpoints.length) {
          if (this.video) {
            const [wraps, time] = this.cpu_time(real_t);
            this.wraps = wraps;
            this.last_time = time;
            this.container!.querySelector("canvas")!.style.display="none";
            this.container!.querySelector("div")!.style.display="none";
          }
          // playback continues
        } else {
          await this.finish_playback(emulator);
        }
        break;
      }
      case ReplayMode.Inactive:
      case ReplayMode.Finished:
        // do nothing
        break;
    }
  }
  static async start_recording(emulator:V86, record_video:boolean, audioDestination:MediaStreamAudioDestinationNode|null, container:HTMLDivElement|null):Promise<Replay> {
    const r = await Replay.create(generateUUID(),ReplayMode.Record);
    r.container = container;
    emulator.v86.cpu.instruction_counter[0] = 0;
    r.make_checkpoint(emulator,new Uint8Array(await emulator.save_state()));
    if (record_video) {
      nonnull(audioDestination);
      nonnull(container);
      const canvas = container.getElementsByTagName("canvas")[0]!;
      const canvas_stream = canvas.captureStream(30);
      const audio_stream = audioDestination.stream;
      const stream = new MediaStream(canvas_stream.getTracks().concat(audio_stream.getTracks()));
      r.recorder = new MediaRecorder(stream, {mimeType:"video/webm"});
      r.recorder.ondataavailable = (e) => {
        r.recording_video.push(e.data);
      };
      r.recorder.start();
    }
    return r;
  }
  private async finish_playback(emulator:V86) {
    emulator.mouse_set_status(true);
    emulator.keyboard_set_status(true);
    this.mode = ReplayMode.Finished;
    if (this.video) {
      await emulator.run();
      const video = this.container!.getElementsByTagName("video")[0]!;
      emulator.speaker_adapter.mixer.set_volume(video.muted ? 0 : 1);
      video.style.display = "none";
      video.pause();
    }
  }
  private async stop_video_recording():Promise<void> {
    if (!this.recorder) { return; }
    const rec = this.recorder;
    this.recorder = null;
    return new Promise((resolve) => {
      rec.onstop = (_e) => {
        this.video = new File(this.recording_video, `replay${this.id}.webm`, {type:"video/webm"});
        this.recording_video = [];
        resolve();
      };
      rec.stop();
    });
  }
  public async get_video():Promise<File|null> {
    return this.video;
  }
  private async finish_recording(emulator:V86) {
    // close file
    this.make_checkpoint(emulator,new Uint8Array(await emulator.save_state()));
    // TODO: await checkpoint encoding?
    if (this.recorder) {
      await this.stop_video_recording();
    }
    this.mode = ReplayMode.Finished;
  }
  async stop(emulator:V86) {
    if(this.mode == ReplayMode.Record) {
      await this.finish_recording(emulator);
    }
    // close file
    if(this.mode == ReplayMode.Playback) {
      await this.finish_playback(emulator);
    }
  }
  async start_playback(emulator:V86, container:HTMLDivElement) {
    this.container = container;
    this.mode = ReplayMode.Playback;
    this.index = 0;
    this.checkpoint_index = 1;
    this.wraps = 0;
    this.last_time = 0;
    emulator.v86.cpu.instruction_counter[0] = 0;
    emulator.mouse_set_status(false);
    emulator.keyboard_set_status(false);
    if (this.video) {
      const video = this.container.getElementsByTagName("video")[0]!;
      video.src = URL.createObjectURL(this.video);
      video.style.display = "block";
      video.play();
      emulator.stop();
    } else {
      const check = this.checkpoints[0];
      const state = await this.restore_checkpoint(check.header_info, check.superblock_seq);
      await emulator.restore_state(state);
    }
  }
  async serialize():Promise<ArrayBuffer> {
    if (this.pending_serialize) { throw "please wait for previous serialize call to finish"; }
    return new Promise((resolve,_reject) => {
      this.pending_serialize = resolve;
      this.worker.postMessage({type:"serialize", args:{events:this.events, checkpoints:this.checkpoints}});
    });
  }
  static async deserialize(buf: ArrayBuffer, video?:File): Promise<Replay> {
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
    if (version <= REPLAY_LEGACY_VERSION) {
      return (await LegacyReplay.deserialize(buf)) as unknown as Replay;
    }
    const r = await Replay.create(id, ReplayMode.Inactive);
    return new Promise((resolve,_reject) => {
      r.pending_deserialize = (data) => {
        r.version = data.version;
        r.events = data.events;
        r.checkpoints = data.checkpoints;
        r.index = 0;
        r.checkpoint_index = 0;
        r.last_time = 0;
        r.wraps = 0;
        r.video = video ?? null;
        resolve(r);
      };
      r.worker.postMessage({type:"deserialize", args:{buffer:buf}},{transfer:[buf]});
    });
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
