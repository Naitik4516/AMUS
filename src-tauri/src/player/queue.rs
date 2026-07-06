use rand::seq::SliceRandom;
use std::collections::VecDeque;

use crate::models::Track;
use super::source::{PlaybackSource, RepeatMode};

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub db_id: i64, // row id in user_queue table, for removal/reorder by id
    pub track: Track,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    track: Track,
    source: PlaybackSource,
}

/// What advancing forward should result in. The actor interprets this;
/// the queue itself never touches the DB.
pub enum NextOutcome {
    Track(Track, PlaybackSource),
    /// Context + user queue both exhausted, repeat is Off, autoplay should
    /// kick in. Actor fetches recommendations and calls `extend_with_autoplay`.
    NeedsAutoplay,
    /// Nothing to play and autoplay is disabled/unavailable.
    End,
}

pub enum PreviousOutcome {
    /// Just restart the current track (elapsed > threshold, or nothing before it).
    RestartCurrent,
    Track(Track, PlaybackSource),
}

pub struct PlaybackQueue {
    context: Vec<Track>,
    context_source: PlaybackSource,
    context_position: Option<usize>, // index into `context` currently "checked out"

    shuffle_enabled: bool,
    shuffle_order: Option<Vec<usize>>, // permutation of context indices
    shuffle_cursor: usize,             // index into shuffle_order

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

    pub fn context_position(&self) -> Option<usize> {
        self.context_position
    }

    pub fn context_len(&self) -> usize {
        self.context.len()
    }

    /// "Up next" preview for UI — user queue first, then a few context tracks.
    pub fn peek_preview(&self, n: usize) -> Vec<Track> {
        let mut out: Vec<Track> = self.user_queue.iter().map(|q| q.track.clone()).collect();
        if out.len() >= n {
            out.truncate(n);
            return out;
        }
        let remaining = n - out.len();
        let upcoming_context_indices = self.upcoming_context_indices(remaining);
        out.extend(upcoming_context_indices.into_iter().map(|i| self.context[i].clone()));
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

    // ---------- loading a new context ----------

    /// Load a fresh album/playlist/artist/favorites/search context and start
    /// playback at `start_index` (index into `tracks` as passed in, i.e.
    /// pre-shuffle order).
    pub fn load_context(&mut self, tracks: Vec<Track>, source: PlaybackSource, start_index: usize) {
        self.context = tracks;
        self.context_source = source;
        self.shuffle_order = None;
        self.shuffle_cursor = 0;
        // Deliberately do NOT clear user_queue: an explicit queue outlives
        // context switches, matching Spotify behavior.
        let start_index = start_index.min(self.context.len().saturating_sub(1));

        if self.shuffle_enabled && !self.context.is_empty() {
            self.regenerate_shuffle_order(Some(start_index));
        }

        self.context_position = if self.context.is_empty() { None } else { Some(start_index) };
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

    /// Replace context with autoplay recommendations once the real context
    /// + user queue are exhausted. Called by the actor after it fetches
    /// `get_similar_tracks`.
    pub fn extend_with_autoplay(&mut self, tracks: Vec<Track>) {
        self.context = tracks;
        self.context_source = PlaybackSource::Recommendations;
        self.shuffle_order = None; // recommendations are already varied, no need to reshuffle
        self.context_position = if self.context.is_empty() { None } else { Some(0) };
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

    /// Fisher-Yates over context indices, with the currently playing index
    /// pinned to the front so toggling shuffle doesn't yank playback elsewhere.
    fn regenerate_shuffle_order(&mut self, pin: Option<usize>) {
        if self.context.is_empty() {
            self.shuffle_order = None;
            return;
        }
        let mut indices: Vec<usize> = (0..self.context.len()).collect();
        if let Some(pin_idx) = pin {
            indices.retain(|&i| i != pin_idx);
        }
        indices.shuffle(&mut rand::rng());   // was: &mut thread_rng()
        if let Some(pin_idx) = pin {
            indices.insert(0, pin_idx);
        }
        self.shuffle_cursor = 0;
        self.shuffle_order = Some(indices);
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    // ---------- user queue (explicit "play next" / "add to queue") ----------

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

    pub fn reorder_queue(&mut self, db_id: i64, new_index: usize) {
        if let Some(idx) = self.user_queue.iter().position(|q| q.db_id == db_id) {
            if let Some(item) = self.user_queue.remove(idx) {
                let new_index = new_index.min(self.user_queue.len());
                self.user_queue.insert(new_index, item);
            }
        }
    }

    // ---------- advance / rewind ----------

    pub fn advance_next(&mut self) -> NextOutcome {
        // push whatever was current onto history before we move away from it
        if let Some((track, source)) = self.current.take() {
            self.history.push(HistoryEntry { track, source });
        }

        // 1. RepeatOne short-circuits everything
        if self.repeat_mode == RepeatMode::One {
            if let Some(entry) = self.history.last() {
                self.current = Some((entry.track.clone(), entry.source.clone()));
                return NextOutcome::Track(entry.track.clone(), entry.source.clone());
            }
        }

        // 2. explicit user queue always wins next
        if let Some(item) = self.user_queue.pop_front() {
            let source = PlaybackSource::Queue;
            self.current = Some((item.track.clone(), source.clone()));
            return NextOutcome::Track(item.track, source);
        }

        // 3. advance within context
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
                    None // ran off the end of the shuffled order
                }
            }
            (None, Some(pos)) => {
                let next = pos + 1;
                if next < self.context.len() { Some(next) } else { None }
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

    /// `elapsed` = seconds into the current track. Standard UX: scrub to
    /// start if we're more than a few seconds in, otherwise actually go back.
    pub fn previous(&mut self, elapsed_sec: f64) -> PreviousOutcome {
        const RESTART_THRESHOLD_SEC: f64 = 3.0;
        if elapsed_sec > RESTART_THRESHOLD_SEC {
            return PreviousOutcome::RestartCurrent;
        }

        match self.history.pop() {
            Some(entry) => {
                // put current back at the front of... nowhere — it just becomes
                // "lost" from context pointer terms; we resume exactly the
                // history entry, context pointer only matters for *forward* nav.
                if let Some((cur_track, cur_source)) = self.current.take() {
                    // if current was a context track, walk context_position back
                    // so a subsequent `next()` doesn't skip it.
                    if cur_source == self.context_source {
                        if let Some(pos) = self.context_position {
                            if pos > 0 && self.context.get(pos) == Some(&cur_track) {
                                // handled implicitly: we're about to set position
                                // to match `entry` below when possible
                            }
                        }
                    }
                }
                // if the history entry is a context track, resync context_position
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(id: i64) -> Track {
        Track {
            id,
            title: format!("Track {id}"),
            artists: vec![],
            album: crate::db::models::Album { id: 1, name: "A".into(), cover_art: None, album_artist: None, year: None },
            duration_seconds: 200,
            is_favorite: false,
            cover_art: None,
            added_at: chrono::Utc::now(),
            track_number: Some(id as u32),
        }
    }

    #[test]
    fn advances_linearly_without_repeat() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![track(1), track(2), track(3)], PlaybackSource::Album(1), 0);
        assert_eq!(q.current().unwrap().0.id, 1);

        match q.advance_next() {
            NextOutcome::Track(t, _) => assert_eq!(t.id, 2),
            _ => panic!(),
        }
        match q.advance_next() {
            NextOutcome::Track(t, _) => assert_eq!(t.id, 3),
            _ => panic!(),
        }
        // exhausted, repeat off -> autoplay
        matches!(q.advance_next(), NextOutcome::NeedsAutoplay);
    }

    #[test]
    fn repeat_all_wraps() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![track(1), track(2)], PlaybackSource::Album(1), 0);
        q.set_repeat(RepeatMode::All);
        q.advance_next(); // -> 2
        match q.advance_next() {
            NextOutcome::Track(t, _) => assert_eq!(t.id, 1), // wrapped
            _ => panic!("expected wrap to track 1"),
        }
    }

    #[test]
    fn repeat_one_replays_same_track() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![track(1), track(2)], PlaybackSource::Album(1), 0);
        q.set_repeat(RepeatMode::One);
        match q.advance_next() {
            NextOutcome::Track(t, _) => assert_eq!(t.id, 1),
            _ => panic!(),
        }
        match q.advance_next() {
            NextOutcome::Track(t, _) => assert_eq!(t.id, 1),
            _ => panic!(),
        }
    }

    #[test]
    fn user_queue_takes_priority_over_context() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![track(1), track(2)], PlaybackSource::Album(1), 0);
        q.enqueue_next(100, track(99));
        match q.advance_next() {
            NextOutcome::Track(t, src) => {
                assert_eq!(t.id, 99);
                assert_eq!(src, PlaybackSource::Queue);
            }
            _ => panic!(),
        }
        // context pointer untouched -> next real advance still goes to track 2
        match q.advance_next() {
            NextOutcome::Track(t, _) => assert_eq!(t.id, 2),
            _ => panic!(),
        }
    }

    #[test]
    fn previous_restarts_if_elapsed_past_threshold() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![track(1), track(2)], PlaybackSource::Album(1), 0);
        q.advance_next();
        matches!(q.previous(10.0), PreviousOutcome::RestartCurrent);
    }

    #[test]
    fn previous_goes_to_history_under_shuffle() {
        let mut q = PlaybackQueue::new();
        q.set_shuffle(true);
        q.load_context(vec![track(1), track(2), track(3)], PlaybackSource::Album(1), 0);
        let first = q.current().unwrap().0.id;
        let second = match q.advance_next() {
            NextOutcome::Track(t, _) => t.id,
            _ => panic!(),
        };
        match q.previous(0.0) {
            PreviousOutcome::Track(t, _) => assert_eq!(t.id, first),
            _ => panic!(),
        }
        let _ = second; // just documenting shuffle order isn't asserted here
    }
}
