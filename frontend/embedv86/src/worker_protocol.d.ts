import {Checkpoint,ReplayEvent} from './v86replay';

interface EncodedCheckpoint {
  which:number;
  header_info:Uint8Array;
  superblock_seq:Uint32Array;
  new_blocks:number[];
  new_superblocks:number[];
}

interface WorkerResponseInitialized {
  type:"initialized";
}
interface WorkerResponseCheckpoint {
  type:"checkpoint";
  args:EncodedCheckpoint;
};
interface WorkerResponseDecodeCheckpoint {
  type:"decode_checkpoint";
  args:{which:number, result:ArrayBuffer};
};
interface WorkerResponseSerialize {
  type: "serialize";
  args:{result:ArrayBuffer}
}
interface WorkerResponseDeserialize {
  type: "deserialize";
  args:{
    id:string,
    version:number,
    events:ReplayEvent[],
    checkpoints:Checkpoint[]
  };
};
export type WorkerResponse = WorkerResponseInitialized | WorkerResponseCheckpoint | WorkerResponseDecodeCheckpoint | WorkerResponseSerialize | WorkerResponseDeserialize;

interface WorkerCommandInit {
  type: "init";
  args: {id:string,version:number};
}
interface WorkerCommandDropCheckpoints {
  type:"drop_checkpoints";
  args:{when:number};
}
interface WorkerCommandSerialize {
  type:"serialize";
  args:{events:ReplayEvent[],checkpoints:Checkpoint[]};
}
interface WorkerCommandDeserialize {
  type:"deserialize";
  args:{buffer:ArrayBuffer};
}
interface WorkerCommandEncode {
  type:"encode";
  args:{time:number, state:Uint8Array, which:number};
}
interface WorkerCommandDecode {
  type:"decode";
  args:{which:number, header_info:Uint8Array, superblock_seq:Uint32Array};
}

export type WorkerCommand = WorkerCommandInit | WorkerCommandDropCheckpoints | WorkerCommandSerialize | WorkerCommandDeserialize | WorkerCommandEncode | WorkerCommandDecode;
