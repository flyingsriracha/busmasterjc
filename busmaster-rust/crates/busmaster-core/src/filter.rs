//! Message filtering for CAN frames
//!
//! This module provides flexible filtering capabilities for CAN messages based on:
//! - ID ranges
//! - ID masks (wildcards)
//! - ID lists
//! - Direction (TX/RX)
//! - Channel
//!
//! # Example
//!
//! ```
//! use busmaster_core::{CanFrame, MessageFilter, FilterRule};
//!
//! // Create a filter that accepts IDs 0x100-0x1FF
//! let filter = MessageFilter::new()
//!     .add_rule(FilterRule::IdRange { start: 0x100, end: 0x1FF });
//!
//! let frame = CanFrame::new_standard(0x150, &[1, 2, 3, 4]).unwrap();
//! assert!(filter.matches(&frame, 0));
//!
//! let frame2 = CanFrame::new_standard(0x200, &[1, 2, 3, 4]).unwrap();
//! assert!(!filter.matches(&frame2, 0));
//! ```

use crate::CanFrame;

/// Direction of a CAN message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Transmitted message
    Tx,
    /// Received message
    Rx,
}

/// A single filter rule
#[derive(Debug, Clone, PartialEq)]
pub enum FilterRule {
    /// Accept messages with IDs in the specified range (inclusive)
    IdRange {
        /// Start of the range (inclusive)
        start: u32,
        /// End of the range (inclusive)
        end: u32,
    },
    /// Accept messages matching the ID mask
    /// The mask specifies which bits to compare (1 = compare, 0 = ignore)
    IdMask {
        /// ID pattern to match
        id: u32,
        /// Mask specifying which bits to compare
        mask: u32,
    },
    /// Accept messages with IDs in the specified list
    IdList {
        /// List of IDs to accept
        ids: Vec<u32>,
    },
    /// Accept messages with the specified direction
    Direction {
        /// Direction to accept
        direction: Direction,
    },
    /// Accept messages on the specified channel
    Channel {
        /// Channel number to accept
        channel: u8,
    },
}

impl FilterRule {
    /// Check if a frame matches this rule
    #[must_use]
    pub fn matches(&self, frame: &CanFrame, channel: u8, direction: Direction) -> bool {
        match self {
            FilterRule::IdRange { start, end } => {
                let id = frame.id();
                id >= *start && id <= *end
            },
            FilterRule::IdMask { id, mask } => {
                let frame_id = frame.id();
                (frame_id & mask) == (id & mask)
            },
            FilterRule::IdList { ids } => {
                let frame_id = frame.id();
                ids.contains(&frame_id)
            },
            FilterRule::Direction { direction: dir } => *dir == direction,
            FilterRule::Channel { channel: ch } => *ch == channel,
        }
    }
}

/// Filter mode determines how multiple rules are combined
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// Accept if ANY rule matches (OR logic)
    Any,
    /// Accept if ALL rules match (AND logic)
    All,
}

/// Message filter for CAN frames
///
/// A filter consists of multiple rules that can be combined using AND or OR logic.
/// By default, the filter uses OR logic (accept if any rule matches).
///
/// # Example
///
/// ```
/// use busmaster_core::{CanFrame, MessageFilter, FilterRule, FilterMode};
///
/// // Create a filter that accepts IDs 0x100-0x1FF on channel 0
/// let filter = MessageFilter::new()
///     .with_mode(FilterMode::All)
///     .add_rule(FilterRule::IdRange { start: 0x100, end: 0x1FF })
///     .add_rule(FilterRule::Channel { channel: 0 });
///
/// let frame = CanFrame::new_standard(0x150, &[1, 2, 3, 4]).unwrap();
/// assert!(filter.matches(&frame, 0));
/// assert!(!filter.matches(&frame, 1)); // Wrong channel
/// ```
#[derive(Debug, Clone)]
pub struct MessageFilter {
    rules: Vec<FilterRule>,
    mode: FilterMode,
}

impl MessageFilter {
    /// Create a new empty filter
    ///
    /// By default, an empty filter accepts all messages.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            mode: FilterMode::Any,
        }
    }

    /// Set the filter mode (AND or OR logic)
    #[must_use]
    pub fn with_mode(mut self, mode: FilterMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a filter rule
    #[must_use]
    pub fn add_rule(mut self, rule: FilterRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Add multiple filter rules
    #[must_use]
    pub fn add_rules(mut self, rules: Vec<FilterRule>) -> Self {
        self.rules.extend(rules);
        self
    }

    /// Check if a frame matches the filter
    ///
    /// # Arguments
    ///
    /// * `frame` - The CAN frame to check
    /// * `channel` - The channel the frame was received on
    ///
    /// # Returns
    ///
    /// `true` if the frame matches the filter, `false` otherwise
    #[must_use]
    pub fn matches(&self, frame: &CanFrame, channel: u8) -> bool {
        self.matches_with_direction(frame, channel, Direction::Rx)
    }

    /// Check if a frame matches the filter with direction
    ///
    /// # Arguments
    ///
    /// * `frame` - The CAN frame to check
    /// * `channel` - The channel the frame was received on
    /// * `direction` - The direction of the frame (TX or RX)
    ///
    /// # Returns
    ///
    /// `true` if the frame matches the filter, `false` otherwise
    #[must_use]
    pub fn matches_with_direction(
        &self,
        frame: &CanFrame,
        channel: u8,
        direction: Direction,
    ) -> bool {
        // Empty filter accepts all messages
        if self.rules.is_empty() {
            return true;
        }

        match self.mode {
            FilterMode::Any => {
                // Accept if ANY rule matches
                self.rules
                    .iter()
                    .any(|rule| rule.matches(frame, channel, direction))
            },
            FilterMode::All => {
                // Accept if ALL rules match
                self.rules
                    .iter()
                    .all(|rule| rule.matches(frame, channel, direction))
            },
        }
    }

    /// Get the number of rules in the filter
    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get the filter mode
    #[must_use]
    pub fn mode(&self) -> FilterMode {
        self.mode
    }

    /// Clear all rules
    pub fn clear(&mut self) {
        self.rules.clear();
    }
}

impl Default for MessageFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter_accepts_all() {
        let filter = MessageFilter::new();
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame, 0));
    }

    #[test]
    fn test_id_range_filter() {
        let filter = MessageFilter::new().add_rule(FilterRule::IdRange {
            start: 0x100,
            end: 0x1FF,
        });

        let frame1 = CanFrame::new_standard(0x100, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame1, 0));

        let frame2 = CanFrame::new_standard(0x150, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame2, 0));

        let frame3 = CanFrame::new_standard(0x1FF, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame3, 0));

        let frame4 = CanFrame::new_standard(0x200, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame4, 0));

        let frame5 = CanFrame::new_standard(0x0FF, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame5, 0));
    }

    #[test]
    fn test_id_mask_filter() {
        // Accept IDs 0x100, 0x101, 0x102, 0x103 (last 2 bits don't matter)
        let filter = MessageFilter::new().add_rule(FilterRule::IdMask {
            id: 0x100,
            mask: 0x7FC, // Compare all bits except last 2
        });

        let frame1 = CanFrame::new_standard(0x100, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame1, 0));

        let frame2 = CanFrame::new_standard(0x101, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame2, 0));

        let frame3 = CanFrame::new_standard(0x103, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame3, 0));

        let frame4 = CanFrame::new_standard(0x104, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame4, 0));
    }

    #[test]
    fn test_id_list_filter() {
        let filter = MessageFilter::new().add_rule(FilterRule::IdList {
            ids: vec![0x100, 0x200, 0x300],
        });

        let frame1 = CanFrame::new_standard(0x100, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame1, 0));

        let frame2 = CanFrame::new_standard(0x200, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame2, 0));

        let frame3 = CanFrame::new_standard(0x150, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame3, 0));
    }

    #[test]
    fn test_channel_filter() {
        let filter = MessageFilter::new().add_rule(FilterRule::Channel { channel: 0 });

        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame, 0));
        assert!(!filter.matches(&frame, 1));
    }

    #[test]
    fn test_direction_filter() {
        let filter = MessageFilter::new().add_rule(FilterRule::Direction {
            direction: Direction::Tx,
        });

        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches_with_direction(&frame, 0, Direction::Tx));
        assert!(!filter.matches_with_direction(&frame, 0, Direction::Rx));
    }

    #[test]
    fn test_multiple_rules_any_mode() {
        let filter = MessageFilter::new()
            .with_mode(FilterMode::Any)
            .add_rule(FilterRule::IdRange {
                start: 0x100,
                end: 0x1FF,
            })
            .add_rule(FilterRule::IdRange {
                start: 0x200,
                end: 0x2FF,
            });

        let frame1 = CanFrame::new_standard(0x150, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame1, 0));

        let frame2 = CanFrame::new_standard(0x250, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame2, 0));

        let frame3 = CanFrame::new_standard(0x300, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame3, 0));
    }

    #[test]
    fn test_multiple_rules_all_mode() {
        let filter = MessageFilter::new()
            .with_mode(FilterMode::All)
            .add_rule(FilterRule::IdRange {
                start: 0x100,
                end: 0x1FF,
            })
            .add_rule(FilterRule::Channel { channel: 0 });

        let frame1 = CanFrame::new_standard(0x150, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame1, 0)); // Matches both rules

        let frame2 = CanFrame::new_standard(0x150, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame2, 1)); // Wrong channel

        let frame3 = CanFrame::new_standard(0x200, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame3, 0)); // Wrong ID range
    }

    #[test]
    fn test_clear_rules() {
        let mut filter = MessageFilter::new().add_rule(FilterRule::IdRange {
            start: 0x100,
            end: 0x1FF,
        });

        assert_eq!(filter.rule_count(), 1);

        let frame = CanFrame::new_standard(0x200, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame, 0));

        filter.clear();
        assert_eq!(filter.rule_count(), 0);
        assert!(filter.matches(&frame, 0)); // Empty filter accepts all
    }

    #[test]
    fn test_extended_id_filter() {
        let filter = MessageFilter::new().add_rule(FilterRule::IdRange {
            start: 0x10000000,
            end: 0x1FFFFFFF,
        });

        let frame1 = CanFrame::new_extended(0x10000000, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame1, 0));

        let frame2 = CanFrame::new_extended(0x15000000, &[1, 2, 3, 4]).unwrap();
        assert!(filter.matches(&frame2, 0));

        let frame3 = CanFrame::new_standard(0x100, &[1, 2, 3, 4]).unwrap();
        assert!(!filter.matches(&frame3, 0));
    }
}
