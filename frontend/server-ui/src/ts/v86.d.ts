// Type definitions with slight modifications retreived from https://github.com/copy/v86/blob/7e6be1e2abfc653052c6ff3f5d45dfe867c7d72b/v86.d.ts

// warning: experimental, incomplete and likely to change in the future
// if you find problems, please send a pull request

/**
 * The type for images as a file, loaded asynchronously.
 *
 * Files with `async: true` and `use_parts: false` are downloaded using HTTP
 * Range requests. Note that not all web servers support this header correctly,
 * and it inherently disables HTTP compression. The `url` fields points
 * directly to the disk images.
 */
type V86AsyncFileImage = {
  /** The URL to the image */
  url: string;

  /**
   * Async loading.
   *
   * If true, the file is downloaded completely, otherwise in chunks
   * (see {@link V86AsyncFileImage.use_parts} for the chunking method).
   */
  async: true;

  /** Image size, required for async loading */
  size?: number;

  /**
   * Use parts instead of Range header, useful for static hostings.
   *
   * If true, v86 expects the image to be split in files of `fixed_chunk_size`
   * bytes. You can use [split-image.py](https://github.com/copy/v86/blob/master/tools/split-image.py)
   * to split an image. V86 appends `-<start-byte>-<end-byte>` to the url.
   */
  use_parts?: boolean;

  /**
   * Fixed chunk size, useful with `use_parts: true` for GitHub Pages users.
   */
  fixed_chunk_size?: number;
};

/**
 * The type for images as a file, loaded synchronously.
 */
type V86SyncFileImage = {
  /** The URL to the image */
  url: string;

  /** @ignore */
  async?: false;
};

/**
 * The type of disk/bios/state images.
 *
 * Note that bios, initial state, bzimage, initrd, multiboot and floppy disk
 * images are always loaded synchronously.
 *
 * State images and fixed-size chunks (but not other image types) that end with
 * .zst are automatically decompressed using a built-in zstd decompressor. This
 * has a performance overhead compared to HTTP compression, but will result in
 * better compression ration.
 */
export type V86Image = V86AsyncFileImage | V86SyncFileImage;
export type V86ImageType = "hda" | "hdb" | "fda" | "fdb" | "cdrom";

/**
 * Emulator instance constructor options.
 */
export interface V86Options {
  /**
   * The memory size in bytes, should be a power of 2.
   * @example 16 * 1024 * 1024
   * @default 64 * 1024 * 1024
   */
  memory_size?: number;

  /**
   * VGA memory size in bytes.
   * @example 8 * 1024 * 1024
   * @default 8 * 1024 * 1024
   */
  vga_memory_size?: number;

  /**
   * BIOS image (supported SeaBIOS and Bochs BIOS)
   * @see {@link https://github.com/copy/v86/tree/master/bios} for BIOS images
   */
  bios?: V86Image;

  /**
   * VGA BIOS image
   * @see {@link https://github.com/copy/v86/tree/master/bios} for BIOS images
   */
  vga_bios?: V86Image;

  /**
   * First hard disk
   */
  hda?: V86Image;

  /**
   * Second hard disk
   */
  hdb?: V86Image;

  /**
   * First floppy disk
   */
  fda?: V86Image;

  /**
   * Second floppy disk
   */
  fdb?: V86Image;

  /**
   * CD-ROM
   * By default, an ejected CD-ROM drive is emulated
   */
  cdrom?: V86Image;
}
