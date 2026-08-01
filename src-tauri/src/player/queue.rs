use rand::seq::SliceRandom;
use std::collections::VecDeque;

use super::source::{PlaybackSource, RepeatMode};
use crate::models::Track;

const MAX_HISTORY: usize = 50;

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

    context_custom_order: Vec<usize>,

    user_queue: VecDeque<QueueItem>,

    history: VecDeque<HistoryEntry>,
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
            context_custom_order: Vec::new(),
            user_queue: VecDeque::new(),
            history: VecDeque::new(),
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
        self.history.back().map(|e| e.track.id)
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
                let start_in_order = self
                    .context_custom_order
                    .iter()
                    .position(|&i| i == pos)
                    .map(|p| p + 1)
                    .unwrap_or(pos + 1);
                let mut cursor = start_in_order;
                while result.len() < n && cursor < self.context_custom_order.len() {
                    result.push(self.context_custom_order[cursor]);
                    cursor += 1;
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
        self.shuffle_cursor = 0;
        self.context_custom_order = (0..self.context.len()).collect();
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
        self.context_custom_order = (0..self.context.len()).collect();
        self.context_position = if self.context.is_empty() {
            None
        } else {
            Some(0)
        };
        self.set_current_from_context();
    }


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

    pub fn remove_from_user_queue(&mut self, db_id: i64) -> Option<QueueItem> {
        let idx = self.user_queue.iter().position(|q| q.db_id == db_id)?;
        self.user_queue.remove(idx)
    }

    pub fn remove_from_context(&mut self, track_id: i64) -> Option<Track> {
        if let Some(idx) = self.context.iter().position(|t| t.id == track_id) {
            let removed_track = self.context.remove(idx);

            // Fix context_custom_order: remove idx and shift higher indices down
            self.context_custom_order.retain(|&i| i != idx);
            for i in &mut self.context_custom_order {
                if *i > idx {
                    *i -= 1;
                }
            }

            // Fix shuffle_order if present
            if let Some(order) = &mut self.shuffle_order {
                // If the removed track was at or before shuffle_cursor, adjust cursor
                if let Some(cursor_pos) = order.iter().position(|&i| i == idx) {
                    if cursor_pos <= self.shuffle_cursor {
                        self.shuffle_cursor = self.shuffle_cursor.saturating_sub(1);
                    }
                }
                order.retain(|&i| i != idx);
                for i in order.iter_mut() {
                    if *i > idx {
                        *i -= 1;
                    }
                }
            }

            if let Some(pos) = self.context_position {
                if pos == idx {
                    self.context_position = None;
                    self.current = None;
                } else if pos > idx {
                    self.context_position = Some(pos - 1);
                }
            }
            Some(removed_track)
        } else {
            None
        }
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

    pub fn reorder_context(&mut self, from_rel: usize, to_rel: usize) {
        if self.shuffle_enabled || self.context.is_empty() {
            return;
        }
        let start_in_order = self
            .context_custom_order
            .iter()
            .position(|&i| i == self.context_position.unwrap_or(0))
            .map(|p| p + 1)
            .unwrap_or(0);
        let from_abs = start_in_order + from_rel;
        let to_abs = start_in_order + to_rel;
        if from_abs < self.context_custom_order.len()
            && to_abs < self.context_custom_order.len()
        {
            let item = self.context_custom_order.remove(from_abs);
            self.context_custom_order.insert(to_abs, item);
        }
    }

    pub fn advance_next(&mut self) -> NextOutcome {
        if let Some((track, source)) = self.current.take() {
            self.history.push_back(HistoryEntry { track, source });
            if self.history.len() > MAX_HISTORY {
                self.history.pop_front();
            }
        }

        if self.repeat_mode == RepeatMode::One {
            if let Some(entry) = self.history.back() {
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
                let next_order_idx = self
                    .context_custom_order
                    .iter()
                    .position(|&i| i == pos)
                    .map(|p| p + 1)
                    .unwrap_or(pos + 1);
                if next_order_idx < self.context_custom_order.len() {
                    Some(self.context_custom_order[next_order_idx])
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

    pub fn jump_to_track(&mut self, track_id: i64) -> bool {
        if let Some(idx) = self.context.iter().position(|t| t.id == track_id) {
            self.context_position = Some(idx);
            if let Some(order) = &self.shuffle_order {
                if let Some(cursor) = order.iter().position(|&i| i == idx) {
                    self.shuffle_cursor = cursor;
                }
            }
            self.set_current_from_context();
            true
        } else {
            false
        }
    }

    pub fn previous(&mut self, elapsed_sec: f64) -> PreviousOutcome {
        const RESTART_THRESHOLD_SEC: f64 = 3.0;
        if elapsed_sec > RESTART_THRESHOLD_SEC {
            return PreviousOutcome::RestartCurrent;
        }

        if self.shuffle_enabled {
            return match self.history.pop_back() {
                Some(entry) => {
                    if entry.source == self.context_source {
                        if let Some(idx) = self.context.iter().position(|t| t.id == entry.track.id)
                        {
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

///////////////////////////////////
///////////  Tests ///////////////
//////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Album, Artist};

    fn make_track(id: i64) -> Track {
        Track {
            id,
            title: format!("Track {}", id),
            artists: vec![Artist {
                id: 1,
                name: "Test Artist".to_string(),
                profile_image: None,
                banner_image: None,
            }],
            album: Album {
                id: 1,
                name: "Test Album".to_string(),
                cover_art: None,
                album_artist: None,
                year: None,
            },
            duration_seconds: 200,
            is_favorite: false,
            cover_art: None,
            added_at: chrono::Utc::now(),
            track_number: Some(id as u32),
            playlist_ids: vec![],
            genre_ids: None,
            queue_id: None,
        }
    }

    #[test]
    fn test_new_queue_is_empty() {
        let q = PlaybackQueue::new();
        assert!(q.current().is_none());
        assert_eq!(q.context_len(), 0);
        assert!(q.user_queue().is_empty());
        assert_eq!(q.repeat_mode(), RepeatMode::Off);
        assert!(!q.shuffle_enabled());
    }

    #[test]
    fn test_load_context_sets_current() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2), make_track(3)];
        q.load_context(tracks.clone(), PlaybackSource::Album(1), 0, None);
        assert_eq!(q.context_len(), 3);
        assert_eq!(q.context_position(), Some(0));
        assert_eq!(q.current().unwrap().0.id, 1);
    }

    #[test]
    fn test_load_context_with_start_index() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2), make_track(3)];
        q.load_context(tracks, PlaybackSource::Album(1), 1, None);
        assert_eq!(q.context_position(), Some(1));
        assert_eq!(q.current().unwrap().0.id, 2);
    }

    #[test]
    fn test_advance_next_through_context() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2), make_track(3)];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);

        let result = q.advance_next();
        assert!(matches!(result, NextOutcome::Track(t, _) if t.id == 2));

        let result = q.advance_next();
        assert!(matches!(result, NextOutcome::Track(t, _) if t.id == 3));
    }

    #[test]
    fn test_advance_next_at_end_without_repeat() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2)];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);

        q.advance_next(); // to track 2
        let result = q.advance_next(); // past end
        assert!(matches!(result, NextOutcome::NeedsAutoplay));
    }

    #[test]
    fn test_advance_next_with_repeat_all() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2)];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);
        q.set_repeat(RepeatMode::All);

        q.advance_next(); // to track 2
        let result = q.advance_next(); // wraps to track 1
        assert!(matches!(result, NextOutcome::Track(t, _) if t.id == 1));
    }

    #[test]
    fn test_advance_next_with_repeat_one() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2)];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);
        q.set_repeat(RepeatMode::One);

        let result = q.advance_next();
        assert!(matches!(result, NextOutcome::Track(t, _) if t.id == 1));
    }

    #[test]
    fn test_enqueue_next_plays_before_context() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![make_track(1)], PlaybackSource::Album(1), 0, None);
        q.enqueue_next(100, make_track(99));

        let result = q.advance_next();
        assert!(matches!(result, NextOutcome::Track(t, _) if t.id == 99));
    }

    #[test]
    fn test_enqueue_end_plays_after_user_queue() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![make_track(1)], PlaybackSource::Album(1), 0, None);
        q.enqueue_end(100, make_track(99));
        q.enqueue_next(101, make_track(98));

        let result = q.advance_next();
        assert!(matches!(result, NextOutcome::Track(t, _) if t.id == 98));
        let result = q.advance_next();
        assert!(matches!(result, NextOutcome::Track(t, _) if t.id == 99));
    }

    #[test]
    fn test_remove_from_queue() {
        let mut q = PlaybackQueue::new();
        q.enqueue_next(1, make_track(10));
        q.enqueue_next(2, make_track(20));

        let removed = q.remove_from_user_queue(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().track.id, 10);
        assert_eq!(q.user_queue().len(), 1);
    }

    #[test]
    fn test_reorder_queue() {
        let mut q = PlaybackQueue::new();
        q.enqueue_end(1, make_track(10));
        q.enqueue_end(2, make_track(20));
        q.enqueue_end(3, make_track(30));

        q.reorder_queue(3, 0);
        assert_eq!(q.user_queue()[0].db_id, 3);
        assert_eq!(q.user_queue()[1].db_id, 1);
    }

    #[test]
    fn test_clear_queue() {
        let mut q = PlaybackQueue::new();
        q.enqueue_next(1, make_track(10));
        q.enqueue_next(2, make_track(20));
        q.clear_queue();
        assert!(q.user_queue().is_empty());
    }

    #[test]
    fn test_jump_to_track_found() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2), make_track(3)];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);

        assert!(q.jump_to_track(3));
        assert_eq!(q.context_position(), Some(2));
        assert_eq!(q.current().unwrap().0.id, 3);
    }

    #[test]
    fn test_jump_to_track_not_found() {
        let mut q = PlaybackQueue::new();
        q.load_context(vec![make_track(1)], PlaybackSource::Album(1), 0, None);
        assert!(!q.jump_to_track(999));
    }

    #[test]
    fn test_previous_past_threshold_restarts() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2)];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);
        q.advance_next(); // now at track 2

        let result = q.previous(5.0); // past threshold
        assert!(matches!(result, PreviousOutcome::RestartCurrent));
    }

    #[test]
    fn test_previous_within_context() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(1), make_track(2)];
        q.load_context(tracks, PlaybackSource::Album(1), 1, None); // start at track 2

        let result = q.previous(1.0); // within threshold, goes back
        assert!(matches!(result, PreviousOutcome::Track(t, _) if t.id == 1));
    }

    #[test]
    fn test_peek_preview_user_queue_first() {
        let mut q = PlaybackQueue::new();
        q.load_context(
            vec![make_track(1), make_track(2)],
            PlaybackSource::Album(1),
            0,
            None,
        );
        q.enqueue_next(100, make_track(99));

        let preview = q.peek_preview(2);
        assert_eq!(preview.len(), 2);
        assert_eq!(preview[0].id, 99); // user queue first
        assert_eq!(preview[1].id, 2); // then upcoming context (skips current track at index 0)
    }

    #[test]
    fn test_set_shuffle_enables_shuffle_order() {
        let mut q = PlaybackQueue::new();
        q.load_context(
            vec![make_track(1), make_track(2), make_track(3)],
            PlaybackSource::Album(1),
            0,
            None,
        );
        assert!(!q.shuffle_enabled());

        q.set_shuffle(true);
        assert!(q.shuffle_enabled());
    }

    #[test]
    fn test_set_shuffle_disabled_clears_shuffle_order() {
        let mut q = PlaybackQueue::new();
        q.load_context(
            vec![make_track(1), make_track(2)],
            PlaybackSource::Album(1),
            0,
            None,
        );
        q.set_shuffle(true);
        q.set_shuffle(false);
        assert!(!q.shuffle_enabled());
    }

    #[test]
    fn test_remove_from_context_shifts_indices() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![
            make_track(10),
            make_track(20),
            make_track(30),
            make_track(40),
        ];
        // Start at track 20
        q.load_context(tracks, PlaybackSource::Album(1), 1, None);
        assert_eq!(q.context_position(), Some(1));
        assert_eq!(q.current().unwrap().0.id, 20);

        // Remove track 30 (index 2) — after current position
        let removed = q.remove_from_context(30);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, 30);
        assert_eq!(q.context_position(), Some(1)); // unchanged
        assert_eq!(q.current().unwrap().0.id, 20);

        // upcoming_context should not panic — only track 40 remains ahead
        let upcoming = q.upcoming_context(10);
        assert_eq!(upcoming.len(), 1);
        assert_eq!(upcoming[0].id, 40);
    }

    #[test]
    fn test_remove_from_context_at_current() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(10), make_track(20), make_track(30)];
        q.load_context(tracks, PlaybackSource::Album(1), 1, None);
        assert_eq!(q.current().unwrap().0.id, 20);

        // Remove the currently playing track
        q.remove_from_context(20);
        assert!(q.current().is_none());
        assert_eq!(q.context_position(), None);

        // Should not panic
        let _ = q.upcoming_context(10);
    }

    #[test]
    fn test_remove_from_context_before_current() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(10), make_track(20), make_track(30)];
        q.load_context(tracks, PlaybackSource::Album(1), 2, None);
        assert_eq!(q.current().unwrap().0.id, 30);

        // Remove track before current position
        q.remove_from_context(10);
        assert_eq!(q.context_position(), Some(1)); // shifted down from 2 to 1
        assert_eq!(q.current().unwrap().0.id, 30);

        // Should not panic
        let _ = q.upcoming_context(10);
    }

    #[test]
    fn test_remove_from_context_last_track() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![make_track(10)];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);

        q.remove_from_context(10);
        assert!(q.current().is_none());
        assert_eq!(q.context_position(), None);
        assert_eq!(q.context_len(), 0);

        // Should not panic
        let _ = q.upcoming_context(10);
    }

    #[test]
    fn test_remove_from_context_with_shuffle() {
        let mut q = PlaybackQueue::new();
        let tracks = vec![
            make_track(10),
            make_track(20),
            make_track(30),
            make_track(40),
            make_track(50),
        ];
        q.load_context(tracks, PlaybackSource::Album(1), 0, None);
        q.set_shuffle(true);

        // Remove a track from the middle
        q.remove_from_context(30);

        // All indices in shuffle_order should be valid
        if let Some(order) = &q.shuffle_order {
            for &i in order {
                assert!(i < q.context.len(), "index {} out of bounds for len {}", i, q.context.len());
            }
        }

        // Should not panic
        let _ = q.upcoming_context(10);
    }
}
