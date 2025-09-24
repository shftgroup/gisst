import { xxhash128 } from "hash-wasm";
import { memcmp } from "./utils";

class Addition {
  when: number;
  first: number;
  public constructor(when:number, first:number) {
    this.when = when;
    this.first = first;
  }
}

export class InsertResult {
  public index:number;
  public is_new:boolean;
  public constructor(index:number, is_new:boolean) {
    this.index = index;
    this.is_new = is_new;
  }
}

export class BlockIndex {
  object_size: number = 8;
  index: Record<string, number[]> = {};
  objects: Uint8Array[] = [];
  counts: number[] = [];
  hashes: string[] = [];
  additions: Addition[] = [];

  public static async create(object_size:number):Promise<BlockIndex> {
    const ths = new BlockIndex();
    ths.object_size = object_size;
    const zeroes = new Uint8Array(object_size);
    if (!await (ths.insert_exact(zeroes, 0, 0))) {
      throw "Couldn't initialize with zeros";
    }
    return ths;
  }
  private bucket_get(bucket:number[], object:Uint8Array): number | undefined {
    for (const which of bucket) {
      if (memcmp(object, this.objects[which])) {
        return which;
      }
    }
    return undefined;
  }
  public async insert(object:Uint8Array, frame:number): Promise<InsertResult> {
    const hash = await xxhash128(new Uint32Array(object.buffer, object.byteOffset, object.byteLength / 4));
    const bucket = this.index[hash];
    if (bucket == undefined) {
      const which = this.objects.length;
      this.index[hash] = [which];
      const copy = object.slice();
      this.objects.push(copy);
      this.hashes.push(hash);
      if (this.additions.length == 0 || this.additions[this.additions.length-1].when < frame) {
        this.additions.push(new Addition(frame, which));
      }
      return new InsertResult(which, true);
    }
    let which = this.bucket_get(bucket, object);
    if (which == undefined) {
      which = this.objects.length;
      bucket.push(which);
      const copy = object.slice();
      this.objects.push(copy);
      this.hashes.push(hash);
      if (this.additions.length == 0 || this.additions[this.additions.length - 1].when < frame) {
        this.additions.push(new Addition(frame, which));
      }
      return new InsertResult(which, true);
    }
    return new InsertResult(which, false);
  }
  public async insert_exact(object:Uint8Array, idx:number, when:number): Promise<boolean> {
    if (idx < this.objects.length) {
      return false;
    }
    const hash = await xxhash128(new Uint32Array(object.buffer, object.byteOffset, object.byteLength / 4));
    const bucket = this.index[hash];
    if (bucket == undefined) {
      this.index[hash] = [idx];
      this.objects.push(object);
      this.hashes.push(hash);
      if (this.additions.length == 0 || this.additions[this.additions.length - 1].when < when) {
        this.additions.push(new Addition(when, idx));
      }
      return true;
    }
    const which = this.bucket_get(bucket, object);
    if (which == undefined) {
      bucket.push(idx);
      this.objects.push(object);
      this.hashes.push(hash);
      if (this.additions.length == 0 || this.additions[this.additions.length - 1].when < when) {
        this.additions.push(new Addition(when, idx));
      }
      return true;
    }
    return false;
  }
  public get(which:number) : Uint8Array {
    return this.objects[which];
  }
  private pop() {
    const idx = this.objects.length-1;
    const object = this.objects.pop();
    const hash = this.hashes.pop();
    if (!hash || !object) {
      throw "Tried to pop empty index";
    }
    const bucket = this.index[hash];
    const loc = bucket.indexOf(idx);
    if (loc < 0) {
      throw "Tried to pop index already popped";
    }
    bucket.splice(loc,1);
    if (bucket.length == 0) {
      delete this.index[hash];
    }
  }
  public remove_after(when:number) {
    let i;
    for (i = this.additions.length-1; i >= 0; i--) {
      const addition = this.additions[i];
      if (addition.when <= when) { break; }
      while (addition.first < this.objects.length) {
        this.pop();
      }
    }
    this.additions.length = i+1;
  }
  public clear() {
    const zeros = this.objects[0];
    this.index = {};
    this.objects.length = 0;
    this.hashes.length = 0;
    this.additions.length = 0;
    this.insert_exact(zeros, 0, 0);
  }
  public length() {
    return this.objects.length;
  }
}

