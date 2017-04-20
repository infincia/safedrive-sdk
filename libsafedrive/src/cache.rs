use std::fs::File;
use std::path::{PathBuf};
use std::io::{Read, Write};
use ::rustc_serialize::hex::{FromHex};
use ::walkdir::WalkDir;

use std::{thread, time};

use ::CACHE_DIR;
use ::block::WrappedBlock;
use ::error::SDError;

use ::models::Token;

use ::error::SDAPIError;

use ::binformat::BinaryWriter;

pub struct WriteCacheMessage {
    pub item: Option<WrappedBlock>,
    pub stop: bool,
    pub error: Option<SDError>,
}

impl WriteCacheMessage {
    pub fn new(item: Option<WrappedBlock>, stop: bool, error: Option<SDError>) -> WriteCacheMessage {
        WriteCacheMessage {
            item: item,
            stop: stop,
            error: error,
        }
    }
}

pub struct WriteCache {
    waiting: Vec<WrappedBlock>,
    item_limit: usize,
    data_limit: usize,
    data_waiting: usize,
    received_final_item: bool,
}


impl WriteCache {
    pub fn new(item_limit: usize, data_limit: usize) -> WriteCache {
        let waiting_list: Vec<WrappedBlock> = Vec::new();

        WriteCache {
            waiting: waiting_list,
            item_limit: item_limit,
            data_limit: data_limit,
            data_waiting: 0,
            received_final_item: false,
        }
    }
    pub fn add(&mut self, block: WrappedBlock) {
        self.data_waiting += block.len();

        self.waiting.push(block);
    }

    #[allow(dead_code)]
    pub fn add_many(&mut self, blocks: Vec<WrappedBlock>) {
        for block in blocks {
            self.data_waiting += block.len();
            self.waiting.push(block);
        }
    }

    pub fn remove(&mut self) -> Option<WrappedBlock> {
        match self.waiting.pop() {
            Some(block) => {
                self.data_waiting -= block.len();

                Some(block)
            },
            None => None,
        }
    }

    pub fn request_waiting_items(&mut self) -> Option<Vec<WrappedBlock>> {
        if self.full() || self.received_final_item {
            let pretty_size_limit = ::util::pretty_bytes(self.data_limit as f64);

            debug!("requesting items from the write cache (limit: {} or {})", self.item_limit, pretty_size_limit);
            let mut r: Vec<WrappedBlock> = Vec::new();

            let mut taken_count = 0;
            let mut taken_size = 0;

            // limit request to `self.desired_group_size` or 100 items, whichever we hit first
            while taken_size < self.data_limit && taken_count < self.item_limit {
                match self.remove() {
                    Some(item) => {
                        taken_size += item.len();
                        taken_count += 1;

                        r.push(item)
                    },
                    None => break,
                }
            }
            let pretty = ::util::pretty_bytes(taken_size as f64);

            debug!("write cache provided {} items ({})", taken_count, pretty);

            return Some(r)
        }

        None
    }

    pub fn received_final_item(&self) -> bool {
        self.received_final_item
    }

    pub fn set_received_final_item(&mut self, value: bool) {
        self.received_final_item = value;
    }

    #[allow(dead_code)]
    pub fn item_limit(&self) -> usize {
        self.item_limit
    }

    #[allow(dead_code)]
    pub fn data_limit(&self) -> usize {
        self.data_limit
    }

    pub fn items_waiting(&self) -> usize {
        self.waiting.len()
    }

    pub fn data_waiting(&self) -> usize {
        self.data_waiting
    }

    pub fn full(&self) -> bool {
        self.data_waiting >= self.data_limit || self.items_waiting() >= self.item_limit
    }

    pub fn upload_thread(self, token: &Token, session_name: &str) -> (::std::sync::mpsc::SyncSender<::cache::WriteCacheMessage>, ::std::sync::mpsc::Receiver<Result<bool, SDError>>) {

        let (block_send, block_receive) = ::std::sync::mpsc::sync_channel::<::cache::WriteCacheMessage>(0);
        let (status_send, status_receive) = ::std::sync::mpsc::channel::<Result<bool, SDError>>();

        let local_token = token.clone();

        let local_session_name = session_name.to_owned();

        let mut local_self = self;

        thread::spawn(move || {
            info!("Running upload thread");

            loop {

                // check to see if we're finished, we need to ensure that we have already put the last
                // block in the cache and that the cache is now empty before exiting
                if local_self.received_final_item() == true && local_self.items_waiting() == 0 {
                    match status_send.send(Ok(true)) {
                        Ok(()) => {
                            let pretty_size = ::util::pretty_bytes(local_self.data_waiting() as f64);
                            debug!("write cache has no more work to do: {} blocks remaining ({})", local_self.items_waiting(), pretty_size);
                            return;
                        },
                        Err(e) => {
                            debug!("channel exited: {}", e);
                            return;
                        },
                    }
                } else {
                    let pretty_size = ::util::pretty_bytes(local_self.data_waiting() as f64);

                    debug!("write cache has more work to do: {} blocks remaining ({})", local_self.items_waiting(), pretty_size);
                }

                // The write cache has both size and item limits, because we need to avoid sending
                // too many blocks or too little data with each request. We try to achieve a balance,
                // sending more blocks if they are very small, fewer if they are very large.
                //
                // For example:
                //
                // A group of 100x 256KiB blocks is 25MiB
                // a group of 100x 256B blocks is 25KiB
                //
                // The tiny group is not worth the HTTP request unless we have no more waiting, but even
                // sending 5x more of the smaller blocks will still only get us a 1MiB request

                match local_self.request_waiting_items() {

                    Some(block_batch) => {

                        if block_batch.len() > 0 {

                            debug!("sending {} blocks", block_batch.len());

                            let block_write_start_time = ::std::time::Instant::now();

                            debug!("write cache trying send of {} blocks", block_batch.len());

                            if ::core::is_sync_task_cancelled(local_session_name.to_owned()) {
                                match status_send.send(Err(SDError::Cancelled)) {
                                    Ok(()) => {
                                        debug!("write thread cancelled");

                                        return;
                                    },
                                    Err(e) => {
                                        debug!("channel exited: {}", e);
                                        break;
                                    },
                                }
                            }

                            match ::sdapi::write_blocks(&local_token, &local_session_name, &block_batch, |total, current, new| {
                                debug!("block upload progress: {}/{}, {} new", current, total, new);
                            }) {
                                Ok(missing) => {
                                    debug!("sending group took {} seconds", block_write_start_time.elapsed().as_secs());

                                    for block in &block_batch {
                                        let bn = &block.name();
                                        if missing.contains(bn) {
                                            debug!("server missing block, returning to the cache: {}", bn);
                                            let mut c = block.clone();
                                            c.needs_upload();
                                            local_self.add(c);
                                        }
                                    }

                                    match status_send.send(Ok(false)) {
                                        Ok(()) => {},
                                        Err(e) => {
                                            debug!("channel exited: {}", e);
                                            break;
                                        },
                                    }
                                },
                                Err(SDAPIError::ServiceUnavailable) => {
                                    match status_send.send(Err(SDError::ServiceUnavailable)) {
                                        Ok(()) => {
                                            debug!("write thread exceeded retries");
                                            return;
                                        },
                                        Err(e) => {
                                            debug!("channel exited: {}", e);
                                            break;
                                        },
                                    }
                                },
                                Err(SDAPIError::Authentication) => {
                                    match status_send.send(Err(SDError::Authentication)) {
                                        Ok(()) => {
                                            debug!("write thread auth failure");
                                            return;
                                        },
                                        Err(e) => {
                                            debug!("channel exited: {}", e);
                                            break;
                                        },
                                    }
                                },
                                Err(e) => {
                                    match status_send.send(Err(SDError::RequestFailure(Box::new(e)))) {
                                        Ok(()) => {
                                            debug!("write thread error");
                                            return;
                                        },
                                        Err(e) => {
                                            debug!("channel exited: {}", e);
                                            break;
                                        },
                                    }
                                },
                            }




                        }

                    },
                    None => {},
                };

                // if the cache is not full, receive a new message from the block iteration thread,
                // this should keep the cache close to full most of the time without stuttering

                if !local_self.full() {
                    let cache_message = match block_receive.try_recv() {
                        Ok(message) => {
                            debug!("write cache: message received");
                            message
                        },
                        Err(e) => {
                            match e {
                                ::std::sync::mpsc::TryRecvError::Empty => {
                                    let delay = time::Duration::from_millis(10);

                                    thread::sleep(delay);
                                    continue;
                                },
                                ::std::sync::mpsc::TryRecvError::Disconnected => {
                                    debug!("write cache: no more messages, end of channel {}", e);
                                    break;
                                },
                            }
                        },
                    };

                    // if we were told to stop processing blocks, just stop immediately
                    if cache_message.stop {
                        debug!("write cache told to stop");

                        return;
                    }

                    debug!("write cache checking item in last message");

                    // check to see if we were given a new block, if so store it in the cache. if we don't
                    // get a block here, it will be because the block iterator thread is completely done
                    // and waiting for this thread to finish uploading them
                    match cache_message.item {
                        Some(block) => {
                            debug!("write cache adding block");

                            local_self.add(block);
                        },
                        None => {
                            debug!("write cache has final item");
                            local_self.set_received_final_item(true);
                        }
                    };
                }
            }

            match status_send.send(Ok(true)) {
                Ok(()) => {
                    debug!("write thread finished, informing main thread to continue");
                },
                Err(e) => {
                    debug!("channel exited: {}", e);
                    return;
                },
            }
        });


        (block_send, status_receive)
    }
}

pub fn clean_cache(limit: u64) -> Result<u64, SDError> {
    let cd = CACHE_DIR.read();
    let bp = PathBuf::from(&*cd);

    let mut estimated_size: u64 = 0;
    debug!("estimating size of cache at {}", bp.display());

    let mut delete_candidates: Vec<(PathBuf, u64, ::std::time::SystemTime)> = Vec::new();

    for block in WalkDir::new(&bp).into_iter().filter_map(|e| e.ok()) {
        let p = block.path();
        debug!("estimating size of {}", p.display());

        let md = match ::std::fs::symlink_metadata(&p) {
            Ok(m) => m,
            Err(_) => { continue },
        };

        let stream_length = md.len();
        let created_date = md.created().expect("block has no creation date");

        estimated_size = estimated_size + stream_length;

        delete_candidates.push((p.to_owned(), stream_length, created_date))
    }

    let size = ::util::pretty_bytes(estimated_size as f64);

    if estimated_size > limit {
        debug!("cache is full ({}), cleaning old blocks", size);

        let to_delete = estimated_size - limit;
        let mut deleted = 0;
        delete_candidates.sort_by(|a, b| {
            a.2.cmp(&b.2)
        });
        while deleted < to_delete {
            if let Some(item) = delete_candidates.pop() {
                let size = item.1;
                match ::std::fs::remove_file(item.0) {
                    Ok(()) => deleted += size,
                    Err(e) => {
                        debug!("block could not be deleted: {}", e);
                    }
                }

            }
        }
        debug!("deleted {}", ::util::pretty_bytes(deleted as f64));

        Ok(deleted)

    } else {
        debug!("cache is not full: {}", size);
        debug!("deleted {}", ::util::pretty_bytes(0 as f64));
        Ok(0)
    }
}

pub fn clear_cache() -> Result<u64, SDError> {
    let cd = CACHE_DIR.read();
    let bp = PathBuf::from(&*cd);

    let mut deleted: u64 = 0;
    debug!("estimating size of cache at {}", bp.display());


    for item in WalkDir::new(&bp).into_iter().filter_map(|e| e.ok()) {
        let p = item.path();

        debug!("deleting item from cache {}", p.display());

        let md = match ::std::fs::symlink_metadata(&p) {
            Ok(m) => m,
            Err(_) => { continue },
        };

        let stream_length = md.len();

        match ::std::fs::remove_file(item.path()) {
            Ok(()) => {},
            Err(e) => {
                debug!("item could not be deleted: {}", e);
            }
        }

        deleted += stream_length;

    }
    debug!("deleted {}", ::util::pretty_bytes(deleted as f64));

    Ok(deleted)
}

pub fn read_block<'a>(name: &'a str) -> Result<WrappedBlock, SDError> {
    let cd = CACHE_DIR.read();
    let mut bp = PathBuf::from(&*cd);

    bp.push(name);

    let mut file = File::open(&bp)?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    let h = name.from_hex().unwrap();

    WrappedBlock::from(buffer, h)

}

pub fn write_binary<'a>(item: &WrappedBlock) -> Result<(), SDError> {
    let cd = CACHE_DIR.read();
    let mut item_path = PathBuf::from(&*cd);
    let name = item.name();

    item_path.push(name);

    let mut f = File::create(&item_path)?;
    f.write_all(&item.as_binary())?;

    Ok(())
}