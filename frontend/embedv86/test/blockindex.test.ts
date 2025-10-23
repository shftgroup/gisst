import { expect, test } from 'vitest'
import { BlockIndex } from '../src/blockindex'

test("init", async () => {
  const idx = await BlockIndex.create(8);
  expect(idx.length()).toBe(1);
  expect(idx.get(0)).toEqual(Uint8Array.from([0,0,0,0,0,0,0,0]));
});

test("insert", async () => {
  const idx = await BlockIndex.create(8);
  const ins1 = await idx.insert(Uint8Array.from([0,0,0,0,0,0,0,1]),1);
  expect(ins1.is_new).toBe(true);
  expect(ins1.index).toBe(1);
  expect(idx.length()).toBe(2);
  const ins1a = await idx.insert(Uint8Array.from([0,0,0,0,0,0,0,1]),1);
  expect(ins1a.is_new).toBe(false);
  expect(ins1a.index).toBe(1);
  expect(idx.length()).toBe(2);
});

test("insert_exact", async () => {
  const idx = await BlockIndex.create(8);
  const ins1 = await idx.insert_exact(Uint8Array.from([0,0,0,0,0,0,0,1]),1,1);
  expect(ins1).toBe(true);
  expect(idx.length()).toBe(2);
  const ins1a = await idx.insert_exact(Uint8Array.from([0,0,0,0,0,0,0,1]),1,1);
  expect(ins1a).toBe(false);
  expect(idx.length()).toBe(2);
  const ins1b = await idx.insert_exact(Uint8Array.from([0,0,0,0,0,0,0,2]),1,1);
  expect(ins1b).toBe(false);
  expect(idx.length()).toBe(2);
});


test("remove_after", async () => {
  const idx = await BlockIndex.create(8);
  await idx.insert(Uint8Array.from([0,0,0,0,0,0,1,1]),1);
  await idx.insert(Uint8Array.from([0,0,0,0,0,0,2,1]),2);
  await idx.insert(Uint8Array.from([0,0,0,0,0,0,2,2]),2);
  await idx.insert(Uint8Array.from([0,0,0,0,0,0,3,1]),3);
  expect(idx.length()).toBe(5);
  idx.remove_after(3);
  expect(idx.length()).toBe(5);
  idx.remove_after(2);
  expect(idx.length()).toBe(4);
  idx.remove_after(0);
  expect(idx.length()).toBe(1);
});
