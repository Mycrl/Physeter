use super::{KernelOptions, Track, Chunk};
use anyhow::Result;
use bytes::{Bytes, BytesMut};
use std::collections::{HashMap, HashSet};
use std::future::Future;

pub struct Previous {
    id: u32,
    track: u16,
    index: u64,
    data: Bytes,
    next: Option<u64>,
    next_track: Option<u16>,
}

pub struct Writer<'a, R> {
    tracks: &'a mut HashMap<u16, Track<'a>>,
    end_callback: fn(u16, u64) -> R,
    callback: fn(u16) -> R,
    options: &'a KernelOptions<'a>,
    write_tracks: HashSet<u16>,
    first_track: Option<u16>,
    first_index: Option<u64>,
    previous: Option<Previous>,
    diff_size: u64,
    buffer: BytesMut,
    track: u16,
    id: u32,
}

impl<'a, R: Future<Output = Result<()>>> Writer<'a, R>{
    pub fn new(
        tracks: &'a mut HashMap<u16, Track<'a>>,
        options: &'a KernelOptions<'_>,
        end_callback: fn(u16, u64) -> R,
        callback: fn(u16) -> R,
    ) -> Self {
        Self {
            end_callback,
            diff_size: options.chunk_size - 17,
            buffer: BytesMut::new(),
            write_tracks: HashSet::new(),
            first_track: None,
            first_index: None,
            previous: None,
            callback,
            options,
            tracks,
            track: 0,
            id: 0,
        }
    }

    async fn alloc(&mut self) -> Result<()> {
        loop {
            if !self.tracks.contains_key(&self.track) {
                (self.callback)(self.track).await?;
                break;
            }

            if let Some(track) = self.tracks.get(&self.track) {
                if track.size + self.options.chunk_size > self.options.track_size {
                    self.track += 1;
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    async fn write_buffer(&mut self, chunk: &[u8], free: bool) -> Result<()> {
        self.buffer.extend_from_slice(chunk);
        if !free && (self.buffer.len() as u64) < self.diff_size {
            return Ok(());
        }

        let mut offset = 0;
        loop {
            let start = offset * self.diff_size;
            let end = start + self.diff_size;

            self.alloc().await?;

            if !self.write_tracks.contains(&self.track) {
                self.write_tracks.insert(self.track);
            }

            let current_track = self.tracks.get_mut(&self.track).unwrap();
            let index = current_track.alloc().await?;

            if let None = self.previous {
                self.first_track = Some(self.track);
                self.first_index = Some(index);
            }

            if let Some(previous) = self.previous.as_mut() {
                let track = self.tracks.get_mut(&previous.track).unwrap();
                track.write(Chunk {
                    id: previous.id,
                    exist: true,
                    next: Some(index),
                    next_track: Some(self.track),
                    data: previous.data.clone()
                }, previous.index).await?;
            }

            let data = &self.buffer[(start as usize)..(end as usize)];
            
            self.previous = Some(Previous {
                data: Bytes::from(data),
                track: self.track,
                next_track: None,
                next: None,
                id: self.id,
                index
            });

            self.id += 1;

            if end as usize >= self.buffer.len() {
                self.buffer.truncate(0);
                break;
            }
        }

        Ok(())
    }
}
