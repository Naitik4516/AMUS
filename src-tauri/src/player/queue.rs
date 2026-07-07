use rand::seq::SliceRandom;
use std::collections::VecDeque;

use super::source::{PlaybackSource, RepeatMode};
use crate::models::Track;

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub db_id: i64,
    pub track: Track,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    track: Track,
    source: PlaybackSource,
}

pub enum NextOutcome {
    Track(Track, PlaybackSource),
    NeedsAutoplay,
    End,
}

pub enum PreviousOutcome {
    RestartCurrent,
    Track(Track, PlaybackSource),
}

pub struct PlaybackQueue {
    context: Vec<Track>,
    context_source: PlaybackSource,
    context_position: Option<usize>, 
    context_label: Option<String>,

    shuffle_enabled: bool,
    shuffle_order: Option<Vec<usize>>,
    shuffle_cursor: usize,            

    user_queue: VecDeque<QueueItem>,

    history: Vec<HistoryEntry>,
    repeat_mode: RepeatMode,

    current: Option<(Track, PlaybackSource)>,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            context: Vec::new(),
            context_source: PlaybackSource::Other,
            context_position: None,
            context_label: None,
            shuffle_enabled: false,
            shuffle_order: None,
            shuffle_cursor: 0,
            user_queue: VecDeque::new(),
            history: Vec::new(),
            repeat_mode: RepeatMode::Off,
            current: None,
        }
    }

    // ---------- accessors ----------

    pub fn current(&self) -> Option<&(Track, PlaybackSource)> {
        self.current.as_ref()
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn shuffle_enabled(&self) -> bool {
        self.shuffle_enabled
    }

    pub fn user_queue(&self) -> &VecDeque<QueueItem> {
        &self.user_queue
    }

    pub fn context_source(&self) -> &PlaybackSource {
        &self.context_source
    }

    pub fn last_played_id(&self) -> Option<i64> {
        self.history.last().map(|e| e.track.id)
    }

    pub fn context_position(&self) -> Option<usize> {
        self.context_position
    }

    pub fn context_len(&self) -> usize {
        self.context.len()
    }

    pub fn peek_preview(&self, n: usize) -> Vec<Track> {
        let mut out: Vec<Track> = self.user_queue.iter().map(|q| q.track.clone()).collect();
        if out.len() >= n {
            out.truncate(n);
            return out;
        }
        let remaining = n - out.len();
        let upcoming_context_indices = self.upcoming_context_indices(remaining);
        out.extend(
            upcoming_context_indices
                .into_iter()
                .map(|i| self.context[i].clone()),
        );
        out
    }

    fn upcoming_context_indices(&self, n: usize) -> Vec<usize> {
        let mut result = Vec::with_capacity(n);
        match (&self.shuffle_order, self.context_position) {
            (Some(order), Some(_)) => {
                let mut cursor = self.shuffle_cursor + 1;
                while result.len() < n && cursor < order.len() {
                    result.push(order[cursor]);
                    cursor += 1;
                }
            }
            (None, Some(pos)) => {
                let mut i = pos + 1;
                while result.len() < n && i < self.context.len() {
                    result.push(i);
                    i += 1;
                }
            }
            _ => {}
        }
        result
    }

    pub fn load_context(
        &mut self,
        tracks: Vec<Track>,
        source: PlaybackSource,
        start_index: usize,
        label: Option<String>,
    ) {
        self.context = tracks;
        self.context_source = source;
        self.context_label = label;
        self.shuffle_order = None;
        // self.shuffle_cursor = 0;
        self.history.clear();
        let start_index = start_index.min(self.context.len().saturating_sub(1));

        if self.shuffle_enabled && !self.context.is_empty() {
            self.regenerate_shuffle_order(Some(start_index));
        }

        self.context_position = if self.context.is_empty() {
            None
        } else {
            Some(start_index)
        };
        self.set_current_from_context();
    }

    fn set_current_from_context(&mut self) {
        if let Some(pos) = self.context_position {
            if let Some(track) = self.context.get(pos) {
                self.current = Some((track.clone(), self.context_source.clone()));
                return;
            }
        }
        self.current = None;
    }

    pub fn extend_with_autoplay(&mut self, tracks: Vec<Track>) {
        self.context = tracks;
        self.context_source = PlaybackSource::Direct;
        self.context_label = None;
        self.shuffle_order = None; // recommendations are already varied
        self.context_position = if self.context.is_empty() {
            None
        } else {
            Some(0)
        };
        self.set_current_from_context();
    }

    // ---------- shuffle ----------

    pub fn set_shuffle(&mut self, enabled: bool) {
        if enabled == self.shuffle_enabled {
            return;
        }
        self.shuffle_enabled = enabled;
        if enabled {
            self.regenerate_shuffle_order(self.context_position);
        } else {
            self.shuffle_order = None;
        }
    }

    pub fn context_label(&self) -> Option<&str> {
        self.context_label.as_deref()
    }

    pub fn upcoming_context(&self, limit: usize) -> Vec<Track> {
        self.upcoming_context_indices(limit)
            .into_iter()
            .map(|i| self.context[i].clone())
            .collect()
    }

    fn regenerate_shuffle_order(&mut self, pin: Option<usize>) {
        if self.context.is_empty() {
            self.shuffle_order = None;
            return;
        }
        let mut indices: Vec<usize> = (0..self.context.len()).collect();
        if let Some(pin_idx) = pin {
            indices.retain(|&i| i != pin_idx);
        }
        indices.shuffle(&mut rand::rng());
        if let Some(pin_idx) = pin {
            indices.insert(0, pin_idx);
        }
        self.shuffle_cursor = 0;
        self.shuffle_order = Some(indices);
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn enqueue_next(&mut self, db_id: i64, track: Track) {
        self.user_queue.push_front(QueueItem { db_id, track });
    }

    pub fn enqueue_end(&mut self, db_id: i64, track: Track) {
        self.user_queue.push_back(QueueItem { db_id, track });
    }

    pub fn remove_from_queue(&mut self, db_id: i64) -> Option<QueueItem> {
        let idx = self.user_queue.iter().position(|q| q.db_id == db_id)?;
        self.user_queue.remove(idx)
    }

    pub fn clear_queue(&mut self) {
        self.user_queue.clear();
    }

    pub fn reorder_queue(&mut self, db_id: i64, new_index: usize) {
        if let Some(idx) = self.user_queue.iter().position(|q| q.db_id == db_id) {
            if let Some(item) = self.user_queue.remove(idx) {
                let new_index = new_index.min(self.user_queue.len());
                self.user_queue.insert(new_index, item);
            }
        }
    }


    pub fn advance_next(&mut self) -> NextOutcome {
        if let Some((track, source)) = self.current.take() {
            self.history.push(HistoryEntry { track, source });
        }

        if self.repeat_mode == RepeatMode::One {
            if let Some(entry) = self.history.last() {
                self.current = Some((entry.track.clone(), entry.source.clone()));
                return NextOutcome::Track(entry.track.clone(), entry.source.clone());
            }
        }

        if let Some(item) = self.user_queue.pop_front() {
            let source = PlaybackSource::Queue;
            self.current = Some((item.track.clone(), source.clone()));
            return NextOutcome::Track(item.track, source);
        }

        self.advance_context()
    }

    fn advance_context(&mut self) -> NextOutcome {
        if self.context.is_empty() {
            return NextOutcome::NeedsAutoplay;
        }

        let next_index = match (&self.shuffle_order, self.context_position) {
            (Some(order), _) => {
                let next_cursor = self.shuffle_cursor + 1;
                if next_cursor < order.len() {
                    self.shuffle_cursor = next_cursor;
                    Some(order[next_cursor])
                } else {
                    None
                }
            }
            (None, Some(pos)) => {
                let next = pos + 1;
                if next < self.context.len() {
                    Some(next)
                } else {
                    None
                }
            }
            (None, None) => Some(0),
        };

        match next_index {
            Some(idx) => {
                self.context_position = Some(idx);
                self.set_current_from_context();
                let track = self.context[idx].clone();
                NextOutcome::Track(track, self.context_source.clone())
            }
            None => match self.repeat_mode {
                RepeatMode::All => {
                    if self.shuffle_enabled {
                        self.regenerate_shuffle_order(None);
                        self.context_position = self.shuffle_order.as_ref().map(|o| o[0]);
                    } else {
                        self.context_position = Some(0);
                    }
                    self.set_current_from_context();
                    let idx = self.context_position.unwrap();
                    NextOutcome::Track(self.context[idx].clone(), self.context_source.clone())
                }
                RepeatMode::Off | RepeatMode::One => NextOutcome::NeedsAutoplay,
            },
        }
    }

    pub fn previous(&mut self, elapsed_sec: f64) -> PreviousOutcome {
        const RESTART_THRESHOLD_SEC: f64 = 3.0;
        if elapsed_sec > RESTART_THRESHOLD_SEC {
            return PreviousOutcome::RestartCurrent;
        }
    
        if self.shuffle_enabled {
            return match self.history.pop() {
                Some(entry) => {
                    if entry.source == self.context_source {
                        if let Some(idx) = self.context.iter().position(|t| t.id == entry.track.id) {
                            self.context_position = Some(idx);
                            if let Some(order) = &self.shuffle_order {
                                if let Some(cursor) = order.iter().position(|&i| i == idx) {
                                    self.shuffle_cursor = cursor;
                                }
                            }
                        }
                    }
                    self.current = Some((entry.track.clone(), entry.source.clone()));
                    PreviousOutcome::Track(entry.track, entry.source)
                }
                None => PreviousOutcome::RestartCurrent,
            };
        }
    
        let currently_from_context = self
            .current
            .as_ref()
            .map(|(_, src)| *src == self.context_source)
            .unwrap_or(false);
    
        if currently_from_context {
            if let Some(pos) = self.context_position {
                if pos > 0 {
                    let new_pos = pos - 1;
                    self.context_position = Some(new_pos);
                    let track = self.context[new_pos].clone();
                    self.current = Some((track.clone(), self.context_source.clone()));
                    return PreviousOutcome::Track(track, self.context_source.clone());
                }
            }
        }
    
        PreviousOutcome::RestartCurrent
    }
}
