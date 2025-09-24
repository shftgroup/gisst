import { assert, expect, test } from 'vitest'
import { BlockIndex } from '../src/blockindex'


test("init", async () => {
  const idx = await BlockIndex.create(16);
  expect(idx.length()).toBe(1);
  expect(idx.get(0)).toEqual(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]));
})
