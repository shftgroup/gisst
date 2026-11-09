use anyhow::{Result, anyhow};
use ffmpeg_next::util::{mathematics::Rescale, rational::Rational};
use ffmpeg_next::{
    format::context::Output as FFOut,
    software::converter as img_conv,
    util::frame::{Audio as FFAFrame, Video as FFVFrame},
};
use retro_rs::Emulator;
use ringbuf::traits::Observer;
use rply_codec::{Frame, decode};
use serde_json::json;
use std::{error::Error, path::Path};

#[derive(Debug, Clone, Copy)]
struct ToI32Err();

impl std::fmt::Display for ToI32Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Float conversion out of integer bounds or applied to nan"
        )
    }
}
impl Error for ToI32Err {}

trait ToI32 {
    fn to_i32(self) -> std::result::Result<i32, ToI32Err>;
}

impl ToI32 for f64 {
    fn to_i32(mut self) -> std::result::Result<i32, ToI32Err> {
        self = self.trunc();
        if self.is_infinite()
            || self.is_nan()
            || (self < f64::from(i32::MIN))
            || (f64::from(i32::MAX) < self)
        {
            return Err(ToI32Err());
        }
        Ok(unsafe { self.to_int_unchecked() })
    }
}

struct VideoState {
    out_video_enc: ffmpeg_next::encoder::video::Encoder,
    out_vframe: FFVFrame,
    out_rgbframe: FFVFrame,
    encoded_video: ffmpeg_next::Packet,
    converter: ffmpeg_next::software::scaling::Context,
    emu_time_base: Rational,
    native_pixel_format: bool,
    stride: usize,
}

impl VideoState {
    fn new(
        emu_time_base: Rational,
        aspect_ratio: Rational,
        w: usize,
        h: usize,
        pixel_format: retro_rs::libretro::retro_pixel_format,
        output: &mut FFOut,
    ) -> Result<Self> {
        let out_video_codec = ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::VP9)
            .ok_or(anyhow!("Could not find output video codec"))?;
        let mut out_video_ctx =
            ffmpeg_next::codec::context::Context::new_with_codec(out_video_codec);
        // out_video_ctx.set_time_base(emu_time_base);
        let mut video_params = ffmpeg_next::codec::Parameters::new();
        unsafe {
            let vps = video_params.as_mut_ptr();
            (*vps).width = i32::try_from(w)?;
            (*vps).height = i32::try_from(h)?;
            (*vps).codec_id = out_video_codec.id().into();
            (*vps).codec_type = ffmpeg_next::ffi::AVMediaType::AVMEDIA_TYPE_VIDEO;
            (*vps).sample_aspect_ratio = aspect_ratio.into();
        };
        out_video_ctx.set_parameters(video_params)?;
        let _out_video = output.add_stream_with(&out_video_ctx)?;
        let encoded_video = ffmpeg_next::Packet::empty();
        // out_video.set_time_base(emu_time_base);
        let mut out_video_enc = out_video_ctx.encoder().video()?;
        out_video_enc.set_format(ffmpeg_next::format::Pixel::YUV420P);
        out_video_enc.set_aspect_ratio(aspect_ratio);
        let w32 = u32::try_from(w)?;
        let h32 = u32::try_from(h)?;
        out_video_enc.set_width(w32);
        out_video_enc.set_height(h32);
        out_video_enc.set_time_base(emu_time_base);
        let out_video_enc = out_video_enc.open()?;
        let out_vframe = FFVFrame::new(
            out_video_enc.format(),
            out_video_enc.width(),
            out_video_enc.height(),
        );
        let (copy_format, is_native, stride) = match pixel_format {
            retro_rs::libretro::retro_pixel_format::RETRO_PIXEL_FORMAT_0RGB1555 => {
                (ffmpeg_next::format::Pixel::RGB555, true, 2)
            }
            retro_rs::libretro::retro_pixel_format::RETRO_PIXEL_FORMAT_XRGB8888 => {
                (ffmpeg_next::format::Pixel::ZRGB32, true, 4)
            }
            retro_rs::libretro::retro_pixel_format::RETRO_PIXEL_FORMAT_RGB565 => {
                (ffmpeg_next::format::Pixel::RGB565, true, 2)
            }
            _other => (ffmpeg_next::format::Pixel::RGB24, false, 3),
        };
        let out_rgbframe = FFVFrame::new(copy_format, w32, h32);

        let converter = img_conv((w32, h32), out_rgbframe.format(), out_video_enc.format())?;
        Ok(Self {
            out_video_enc,
            out_vframe,
            out_rgbframe,
            encoded_video,
            converter,
            emu_time_base,
            native_pixel_format: is_native,
            stride,
        })
    }
    fn writeout(&mut self, output: &mut FFOut) -> Result<()> {
        let output_time_base = output
            .stream(0)
            .ok_or(anyhow!("no video stream"))?
            .time_base();
        while self
            .out_video_enc
            .receive_packet(&mut self.encoded_video)
            .is_ok()
        {
            self.encoded_video.set_stream(0);
            self.encoded_video
                .rescale_ts(self.out_video_enc.time_base(), output_time_base);
            self.encoded_video.write_interleaved(output)?;
        }
        Ok(())
    }
    fn send_frame(&mut self, emu: &Emulator, frame_num: u64, output: &mut FFOut) -> Result<()> {
        // output one frame of video/audio, set_pts
        // copy video to out_vframe
        if self.native_pixel_format {
            let pitch = emu.framebuffer_pitch();
            let (w, h) = emu.framebuffer_size();
            let stride = self.stride;
            emu.peek_framebuffer(|fb| {
                let data = self.out_rgbframe.data_mut(0);
                for y in 0..h {
                    data[(y * w * stride)..((y + 1) * w * stride)]
                        .copy_from_slice(&fb[(y * pitch)..(y * pitch + w * stride)]);
                }
            })?;
        } else {
            emu.copy_framebuffer_rgb888(self.out_rgbframe.data_mut(0))?;
        }
        self.converter
            .run(&self.out_rgbframe, &mut self.out_vframe)?;
        let frame_num = i64::try_from(frame_num)?;
        let frame_pts = frame_num.rescale(self.emu_time_base, self.out_video_enc.time_base());
        self.out_vframe.set_pts(Some(frame_pts));
        self.out_video_enc.send_frame(&self.out_vframe)?;
        self.writeout(output)
    }
    fn drain(&mut self, output: &mut FFOut) -> Result<()> {
        self.out_video_enc.send_eof()?;
        self.writeout(output)
    }
}

struct AudioState {
    out_audio_enc: ffmpeg_next::encoder::audio::Encoder,
    out_aframe: FFAFrame,
    in_aframe: FFAFrame,
    encoded_audio: ffmpeg_next::Packet,
    audio_buf: ringbuf::LocalRb<ringbuf::storage::Heap<i16>>,
    audio_frame_out: i64,
    audio_frame_in: i64,
    resampler: ffmpeg_next::software::resampling::Context,
}

impl AudioState {
    fn new(in_audio_sample_rate: i32, output: &mut FFOut) -> Result<Self> {
        let out_audio_codec = ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::OPUS)
            .ok_or(anyhow!("Could not find output audio codec"))?;
        let mut out_audio_ctx =
            ffmpeg_next::codec::context::Context::new_with_codec(out_audio_codec);
        // out_audio_ctx.debug(Debug::all());
        let mut audio_params = ffmpeg_next::codec::Parameters::new();
        unsafe {
            let aps = audio_params.as_mut_ptr();
            ffmpeg_next::ffi::av_channel_layout_default(&raw mut (*aps).ch_layout, 2);
            (*aps).codec_id = out_audio_codec.id().into();
            (*aps).codec_type = ffmpeg_next::ffi::AVMediaType::AVMEDIA_TYPE_AUDIO;
            (*aps).sample_rate = 48000;
            (*aps).frame_size = 960;
            (*aps).bit_rate = 128000;
        };
        out_audio_ctx
            .set_parameters(audio_params)
            .map_err(|e| anyhow!("could not set params {e:?}"))?;
        let _out_audio = output
            .add_stream_with(&out_audio_ctx)
            .map_err(|e| anyhow!("could not add stream {e:?}"))?;
        let encoded_audio = ffmpeg_next::Packet::empty();
        let audio_time_base = Rational::new(1, 48000);
        let mut out_audio_enc = out_audio_ctx
            .encoder()
            .audio()
            .map_err(|e| anyhow!("could not obtain audio encoder {e:?}"))?;
        out_audio_enc.set_format(ffmpeg_next::format::Sample::F32(
            ffmpeg_next::format::sample::Type::Packed,
        ));
        out_audio_enc.set_channel_layout(ffmpeg_next::ChannelLayout::STEREO);
        out_audio_enc.set_time_base(audio_time_base);
        out_audio_enc.set_bit_rate(128000);
        out_audio_enc.set_rate(audio_time_base.1);
        let out_audio_enc = out_audio_enc
            .open()
            .map_err(|e| anyhow!("could not open audio encoder {e:?}"))?;
        let mut in_aframe = FFAFrame::new(
            ffmpeg_next::format::Sample::I16(ffmpeg_next::format::sample::Type::Packed),
            1024, // Just my choice of buffering rate
            ffmpeg_next::ChannelLayout::STEREO,
        );
        in_aframe.set_rate(u32::try_from(in_audio_sample_rate)?);
        let mut out_aframe = FFAFrame::new(
            out_audio_enc.format(),
            out_audio_enc.frame_size() as usize,
            out_audio_enc.channel_layout(),
        );
        out_aframe.set_rate(out_audio_enc.rate());
        let resampler = in_aframe.resampler(
            out_aframe.format(),
            out_aframe.channel_layout(),
            out_aframe.rate(),
        )?;
        let audio_buf = ringbuf::LocalRb::new(in_aframe.samples() * 2 * 20);

        Ok(Self {
            out_audio_enc,
            out_aframe,
            encoded_audio,
            audio_buf,
            audio_frame_out: 0,
            audio_frame_in: 0,
            resampler,
            in_aframe,
        })
    }
    fn writeout(&mut self, output: &mut FFOut) -> Result<()> {
        let output_time_base = output
            .stream(1)
            .ok_or(anyhow!("Missing audio output stream"))?
            .time_base();
        while self
            .out_audio_enc
            .receive_packet(&mut self.encoded_audio)
            .is_ok()
        {
            self.encoded_audio.set_stream(1);
            self.encoded_audio
                .rescale_ts(self.out_audio_enc.time_base(), output_time_base);
            self.encoded_audio.write_interleaved(output)?;
        }
        Ok(())
    }
    fn resample_and_send(&mut self, output: &mut FFOut, drain: bool) -> Result<()> {
        // if there are not enough samples to hit 960 bytes, just do nothing
        let can_run = self.resampler.delay().is_some_and(|d| d.output >= 480);
        if can_run {
            self.resampler.run(&self.in_aframe, &mut self.out_aframe)?;
            self.in_aframe.set_pts(Some(self.audio_frame_in));
            self.out_aframe.set_pts(Some(self.audio_frame_out));
            self.audio_frame_in += i64::try_from(self.in_aframe.samples())?;
            self.audio_frame_out += i64::try_from(self.out_aframe.samples())?;
            self.out_audio_enc.send_frame(&self.out_aframe)?;
            self.writeout(output)?;
        }
        while let Some(delay) = self.resampler.delay()
            && (delay.output >= 480 || drain)
        {
            self.in_aframe.set_pts(Some(self.audio_frame_in));
            self.out_aframe.set_pts(Some(self.audio_frame_out));
            self.resampler.flush(&mut self.out_aframe)?;
            self.audio_frame_in += i64::try_from(self.in_aframe.samples())?;
            self.audio_frame_out += i64::try_from(self.out_aframe.samples())?;
            self.out_audio_enc.send_frame(&self.out_aframe)?;
            self.writeout(output)?;
        }
        Ok(())
    }
    fn send_frames(&mut self, emu: &Emulator, output: &mut FFOut) -> Result<()> {
        emu.peek_audio_sample(|samples| {
            self.audio_buf.push_slice_overwrite(samples);
            while self.audio_buf.occupied_len() >= self.in_aframe.samples() * 2 {
                let (_, toconvert, _) = unsafe { self.in_aframe.data_mut(0).align_to_mut::<i16>() };
                let avail = self.audio_buf.pop_slice(toconvert);
                if avail != toconvert.len() {
                    return Err(anyhow!(
                        "popped slice was wrong length, {avail} instead of {}",
                        toconvert.len()
                    ));
                }
                self.resample_and_send(output, false)?;
            }
            Ok(())
        })
    }
    fn drain(&mut self, output: &mut FFOut) -> Result<()> {
        while self.audio_buf.occupied_len() > 0 {
            let (_, toconvert, _) = unsafe { self.in_aframe.data_mut(0).align_to_mut::<i16>() };
            let len = self.audio_buf.pop_slice(toconvert);
            toconvert[len..].fill(0);
            self.resample_and_send(output, true)?;
        }
        self.resample_and_send(output, true)?;
        self.out_audio_enc.send_eof()?;
        self.writeout(output)
    }
}

fn upgrade_v0<R: std::io::BufRead + 'static>(
    emu: &mut Emulator,
    mut rply: rply_codec::ReplayDecoder<R>,
    tmp_path: &Path,
) -> Result<()> {
    use rply_codec::InputData;
    use std::cell::RefCell;
    use std::rc::Rc;
    let mut writer = std::io::BufWriter::new(std::fs::File::create(&tmp_path)?);
    assert!(emu.load(&rply.initial_state));
    let mut header_out = rply.header.clone();
    header_out.upgrade();
    let mut encoder = rply_codec::encode(header_out, &rply.initial_state, &mut writer).unwrap();
    let mut frame = Frame::default();
    rply.read_key_events(&mut frame).unwrap();
    rply.read_end_of_frame(&mut frame).unwrap();
    let frame = Rc::new(RefCell::new(frame));
    let rply = Rc::new(RefCell::new(rply));
    let cb = {
        let frame = Rc::clone(&frame);
        let rply = Rc::clone(&rply);
        Box::new(move |port, device, idx, id| {
            let val = rply.borrow_mut().read_v0_button().unwrap();
            //println!("{port}-{device}-{idx}-{id}: 0x{val:x}");
            frame.borrow_mut().input_events.push(InputData {
                port: u8::try_from(port).unwrap(),
                device: u8::try_from(device).unwrap(),
                idx: u8::try_from(idx).unwrap(),
                id: u16::try_from(id).unwrap(),
                val,
            });
            val
        })
    };
    let mut i = 0;
    loop {
        frame.borrow_mut().clear();
        emu.run_with_button_callback(cb.clone());
        match rply.borrow_mut().read_key_events(&mut frame.borrow_mut()) {
            Ok(()) => {}
            Err(rply_codec::ReplayError::IO(e)) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(anyhow!(e));
            }
            Err(e) => return Err(anyhow!(e)),
        }
        rply.borrow_mut()
            .read_end_of_frame(&mut frame.borrow_mut())
            .unwrap();
        encoder.write_frame(&frame.borrow()).unwrap();
        println!(
            "{}",
            json!({"status":"progress", "phase": "upgrade", "frame": i, "total":null})
        );
        i += 1;
    }
    encoder.finish()?;
    Ok(())
}

// bobl example: cargo run --bin genvideo examples/bobl.replay examples/bobl.mp4 cores/fceumm_libretro roms/bobl.nes
// ff3 example: cargo run --bin genvideo examples/ff3v2.replay examples/ff3.mp4 cores/snes9x_libretro roms/ff3.nes
#[allow(clippy::too_many_lines)]
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        println!(
            "{}",
            json!({"status":"error", "error":"Usage error","cause":format!("wrong number of arguments {}",args.len())})
        );
        std::process::exit(-20);
    }
    let replay_file = args.get(1).unwrap();
    let file =
        std::fs::File::open(replay_file).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"Failed to read replay file", "cause":format!("{e:#}")}));
            std::process::exit(-1);
        });
    let outfile = std::path::PathBuf::from(args.get(2).unwrap());
    let corefile = args.get(3).unwrap().clone();
    let core_full_path = &std::env::current_dir()
        .unwrap_or_else(|e| {
            println!(
                "{}",
                json!({"status":"error", "error":"Failed to get cwd", "cause":format!("{e:#}")})
            );
            std::process::exit(-1);
        })
        .join(&corefile);
    let romfile = args.get(4).unwrap().clone();
    if let Err(e) = ffmpeg_next::init() {
        println!(
            "{}",
            json!({"status":"error", "error":"Failed to initialize ffmpeg", "cause":format!("{e:#}")})
        );
        std::process::exit(-10);
    }
    ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Quiet);
    // TODO: handle other files like dependencies?  chdir to system path?
    let mut emu = Emulator::create_with_gfx(
        core_full_path,
        Path::new(&romfile),
        Box::new(retro_rs::GlGfx::default()),
    );
    let mut rply = decode(std::io::BufReader::new(file)).unwrap_or_else(|e| {
        println!(
            "{}",
            json!({"status":"error", "error":"Failed to decode replay","cause":format!("{e:#}")})
        );
        std::process::exit(-1);
    });
    rply = if rply.header.version() == 0 {
        let temp = std::env::temp_dir().join("ra-video-genvideo");
        std::fs::create_dir_all(&temp).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"failed to create temporary output directory","cause":format!("{e:#}")}));
            std::process::exit(-41);
        });
        let tmp_path = temp.join(format!("{}.v2.replay", rply.header.identifier()));
        upgrade_v0(&mut emu, rply, &tmp_path).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"failed to upgrade v0 replay","cause":format!("{e:#}")}));
            std::process::exit(-41);
        });
        let upgraded_file = std::fs::File::open(&tmp_path).unwrap_or_else(|e| {
                        println!(
                "{}",
                json!({"status":"error", "error":"Failed to open upgraded replay","cause":format!("{e:#}")})
            );
            std::process::exit(-42);

        });
        decode(std::io::BufReader::new(upgraded_file)).unwrap_or_else(|e| {
            println!(
                "{}",
                json!({"status":"error", "error":"Failed to decode upgraded replay","cause":format!("{e:#}")})
            );
            std::process::exit(-43);
        })
    } else {
        rply
    };
    let frame_count = rply.header.frame_count();
    // run emu a tick to make sure we have right frame sizes, etc
    emu.run([retro_rs::Buttons::default(); 2]);
    let (w, h) = emu.framebuffer_size();
    let pixel_format = emu.pixel_format();
    if !emu.load(&rply.initial_state) {
        println!(
            "{}",
            json!({"status":"error", "error":"replay initial state did not load","cause":""})
        );
        std::process::exit(-2);
    }
    let mut output = ffmpeg_next::format::output(&outfile).unwrap_or_else(|e| {
        println!(
            "{}",
            json!({"status":"error", "error":"Failed to open output file","cause":format!("{e:#}")})
        );
        std::process::exit(-11);
    });
    let emu_video_framerate = emu.get_video_fps().to_i32().unwrap_or_else(|e| {
        println!(
            "{}",
            json!({"status":"error", "error":"unsupported video fps","cause":format!("{e:#}")})
        );
        std::process::exit(-23);
    });
    let emu_time_base = Rational::new(1, emu_video_framerate);
    let audio_sample_rate = emu.get_audio_sample_rate().to_i32().unwrap_or_else(|e| {
        println!("{}",json!({"status":"error", "error":"unsupported audio samplerate","cause":format!("{e:#}")}));
        std::process::exit(-23);
    });
    let aspect_ratio = Rational::from(f64::from(emu.get_aspect_ratio()));
    let mut video_state =
        VideoState::new(emu_time_base, aspect_ratio, w, h, pixel_format, &mut output).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"failed to create video state","cause":format!("{e:#}")}));
            std::process::exit(-11);
        });
    let mut audio_state = AudioState::new(audio_sample_rate, &mut output).unwrap_or_else(|e| {
        for e in e.chain() {
            println!("E:{e:?}");
        }
        println!("{}",json!({"status":"error", "error":"failed to create audio state","cause":format!("{e:#}")}));
        std::process::exit(-11);
    });
    output.write_header().unwrap_or_else(|e| {
        println!("{}",json!({"status":"error", "error":"Failed to write output file header","cause":format!("{e:#}")}));
        std::process::exit(-11);
    });
    // video_state
    //     .encoded_video
    //     .set_time_base(video_stream_time_base);
    // audio_state
    //     .encoded_audio
    //     .set_time_base(audio_stream_time_base);

    let mut frame = Frame::default();
    loop {
        match rply.read_frame(&mut frame) {
            Err(rply_codec::ReplayError::IO(_)) if frame_count.is_none() => break,
            Err(e) => {
                println!(
                    "{}",
                    json!({"status":"error", "error":"replay read error","cause":format!("{e:#}")})
                );
                std::process::exit(-12);
            }
            _ => (),
        }
        let buttons = frame_to_buttons(&frame);
        emu.run(buttons);
        video_state.send_frame(&emu, rply.frame_number, &mut output).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"Failed to send video frame","cause":format!("{e:#}")}));
            std::process::exit(-12);
        });
        audio_state.send_frames(&emu, &mut output).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"Failed to send audio frame","cause":format!("{e:#}")}));
            std::process::exit(-12);
        });
        if !frame.checkpoint_bytes.is_empty() && !emu.load(&frame.checkpoint_bytes) {
            println!(
                "{}",
                json!({"status":"error", "error":"replay checkpoint did not load", "cause":{"frame":rply.frame_number, "total":frame_count}})
            );
            std::process::exit(-2);
        }
        println!(
            "{}",
            json!({"status":"progress", "phase": "render", "frame":rply.frame_number, "total":frame_count})
        );

        if Some(rply.frame_number) == frame_count {
            break;
        }
    }
    if frame_count.is_some_and(|fc| fc != rply.frame_number) {
        println!(
            "{}",
            json!({"status":"error", "error":"replay did not finish properly", "cause":{"frame":rply.frame_number, "total":frame_count}})
        );
        std::process::exit(-3);
    }
    audio_state.drain(&mut output).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"Failed to drain audio state","cause":format!("{e:#}")}));
            std::process::exit(-13);
        });
    video_state.drain(&mut output).unwrap_or_else(|e| {
            println!("{}",json!({"status":"error", "error":"Failed to drain video state","cause":format!("{e:#}")}));
            std::process::exit(-13);
        });
    output.write_trailer().unwrap_or_else(|e| {
        println!("{}",json!({"status":"error", "error":"Failed to write output trailer","cause":format!("{e:#}")}));
        std::process::exit(-14);
    });
    println!("{}", json!({"status":"finished"}));
}

fn frame_to_buttons(frame: &Frame) -> [retro_rs::Buttons; 2] {
    use retro_rs::Buttons;
    let mut buttons = [0_i16; 2];
    for inp in &frame.input_events {
        let port = usize::from(inp.port);
        if port < buttons.len() && inp.device == 1 {
            buttons[port] |= inp.val;
        }
    }
    [Buttons::from(buttons[0]), Buttons::from(buttons[1])]
}
