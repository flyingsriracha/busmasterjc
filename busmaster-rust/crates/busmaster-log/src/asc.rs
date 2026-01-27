//! ASC (ASCII) log file format writer.
//!
//! The ASC format is a human-readable text format used by Vector CANoe/CANalyzer.
//!
//! ## Format Structure
//!
//! ```text
//! date Fri Dec 9 11:58:31 am 2011
//! base hex  timestamps absolute
//! no internal events logged
//! // version 7.1.0
//! 0.001250 1  123             Tx   d 8 00 01 02 03 04 05 06 07
//! 0.002500 1  456x            Rx   d 4 AA BB CC DD
//! End TriggerBlock
//! ```
//!
//! ## Example
//!
//! ```no_run
//! use busmaster_log::AscWriter;
//! use busmaster_core::CanFrame;
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut writer = AscWriter::create("output.asc")?;
//!
//! let frame = CanFrame::new_standard(0x123, &[0x00, 0x01, 0x02, 0x03])?;
//! writer.log_frame(&frame, Duration::from_millis(1250), 1, true)?;
//!
//! writer.close()?;
//! # Ok(())
//! # }
//! ```

use busmaster_core::{CanFrame, Result};
use chrono::{Datelike, Local, Timelike};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Duration;

/// ASC log file writer.
///
/// Writes CAN frames to an ASC format log file compatible with Vector CANoe/CANalyzer.
pub struct AscWriter {
    writer: BufWriter<File>,
}

impl AscWriter {
    /// Creates a new ASC log file and writes the header.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output ASC file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use busmaster_log::AscWriter;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let writer = AscWriter::create("output.asc")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let start_time = Local::now();

        // Write ASC header
        Self::write_header(&mut writer, &start_time)?;

        Ok(Self { writer })
    }

    /// Writes the ASC file header.
    fn write_header(
        writer: &mut BufWriter<File>,
        timestamp: &chrono::DateTime<Local>,
    ) -> Result<()> {
        // Format: date Fri Dec 9 11:58:31 am 2011
        let weekday = match timestamp.weekday() {
            chrono::Weekday::Mon => "Mon",
            chrono::Weekday::Tue => "Tue",
            chrono::Weekday::Wed => "Wed",
            chrono::Weekday::Thu => "Thu",
            chrono::Weekday::Fri => "Fri",
            chrono::Weekday::Sat => "Sat",
            chrono::Weekday::Sun => "Sun",
        };

        let month = match timestamp.month() {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "Unknown",
        };

        let hour = timestamp.hour();
        let (hour_12, am_pm) = if hour == 0 {
            (12, "am")
        } else if hour < 12 {
            (hour, "am")
        } else if hour == 12 {
            (12, "pm")
        } else {
            (hour - 12, "pm")
        };

        writeln!(
            writer,
            "date {} {} {} {}:{}:{} {} {}",
            weekday,
            month,
            timestamp.day(),
            hour_12,
            timestamp.minute(),
            timestamp.second(),
            am_pm,
            timestamp.year()
        )?;

        writeln!(writer, "base hex  timestamps absolute")?;
        writeln!(writer, "no internal events logged")?;
        writeln!(writer, "// version 7.1.0")?;

        Ok(())
    }

    /// Logs a CAN frame to the ASC file.
    ///
    /// # Arguments
    ///
    /// * `frame` - The CAN frame to log
    /// * `timestamp` - Time since logging started
    /// * `channel` - CAN channel number (1-based)
    /// * `is_tx` - true for transmitted frames, false for received
    ///
    /// # Format
    ///
    /// ```text
    /// <timestamp> <channel> <id> <dir> d <dlc> <data bytes>
    /// ```
    ///
    /// Example: `0.001250 1  123             Tx   d 8 00 01 02 03 04 05 06 07`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use busmaster_log::AscWriter;
    /// use busmaster_core::CanFrame;
    /// use std::time::Duration;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut writer = AscWriter::create("output.asc")?;
    /// let frame = CanFrame::new_standard(0x123, &[0x00, 0x01, 0x02, 0x03])?;
    ///
    /// writer.log_frame(&frame, Duration::from_millis(1250), 1, true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn log_frame(
        &mut self,
        frame: &CanFrame,
        timestamp: Duration,
        channel: u8,
        is_tx: bool,
    ) -> Result<()> {
        // Convert timestamp to seconds with microsecond precision
        let secs = timestamp.as_secs();
        let micros = timestamp.subsec_micros();
        let timestamp_str = format!("{}.{:06}", secs, micros);

        // Format ID with 'x' suffix for extended frames
        let id_str = if frame.is_extended() {
            format!("{}x", frame.id())
        } else {
            format!("{}", frame.id())
        };

        // Direction: Tx or Rx
        let dir = if is_tx { "Tx" } else { "Rx" };

        // Format data bytes as hex
        let data_str = frame
            .data()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        // Write frame line
        // Format: <timestamp> <channel> <id> <dir> d <dlc> <data>
        writeln!(
            self.writer,
            "{} {}  {:<16} {}   d {} {}",
            timestamp_str,
            channel,
            id_str,
            dir,
            frame.dlc(),
            data_str
        )?;

        Ok(())
    }

    /// Flushes any buffered data to the file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use busmaster_log::AscWriter;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut writer = AscWriter::create("output.asc")?;
    /// // ... log some frames ...
    /// writer.flush()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    /// Closes the ASC file, writing the end marker and flushing all data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use busmaster_log::AscWriter;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut writer = AscWriter::create("output.asc")?;
    /// // ... log some frames ...
    /// writer.close()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn close(mut self) -> Result<()> {
        writeln!(self.writer, "End TriggerBlock")?;
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_asc_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let writer = AscWriter::create(path).unwrap();
        drop(writer);

        // Verify file was created and has header
        let mut content = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("date"));
        assert!(content.contains("base hex  timestamps absolute"));
        assert!(content.contains("no internal events logged"));
        assert!(content.contains("// version 7.1.0"));
    }

    #[test]
    fn test_log_standard_frame() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut writer = AscWriter::create(path).unwrap();
        let frame = CanFrame::new_standard(0x123, &[0x00, 0x01, 0x02, 0x03]).unwrap();

        writer
            .log_frame(&frame, Duration::from_millis(1250), 1, true)
            .unwrap();
        writer.close().unwrap();

        // Verify frame was logged
        let content = fs::read_to_string(path).unwrap();
        println!("Content:\n{}", content);
        assert!(content.contains("1.250000"));
        assert!(content.contains("291")); // 0x123 in decimal
        assert!(content.contains("Tx"));
        assert!(content.contains("d 4"));
        assert!(content.contains("00 01 02 03"));
        assert!(content.contains("End TriggerBlock"));
    }

    #[test]
    fn test_log_extended_frame() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut writer = AscWriter::create(path).unwrap();
        let frame = CanFrame::new_extended(0x12345678, &[0xAA, 0xBB, 0xCC, 0xDD]).unwrap();

        writer
            .log_frame(&frame, Duration::from_millis(2500), 2, false)
            .unwrap();
        writer.close().unwrap();

        // Verify extended frame was logged with 'x' suffix
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("2.500000"));
        assert!(content.contains("305419896x")); // 0x12345678 in decimal with 'x'
        assert!(content.contains("Rx"));
        assert!(content.contains("d 4"));
        assert!(content.contains("AA BB CC DD"));
    }

    #[test]
    fn test_log_multiple_frames() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut writer = AscWriter::create(path).unwrap();

        // Log multiple frames
        let frame1 = CanFrame::new_standard(0x100, &[0x11, 0x22]).unwrap();
        let frame2 = CanFrame::new_standard(0x200, &[0x33, 0x44, 0x55]).unwrap();
        let frame3 = CanFrame::new_extended(0x300, &[0x66, 0x77, 0x88, 0x99]).unwrap();

        writer
            .log_frame(&frame1, Duration::from_millis(100), 1, true)
            .unwrap();
        writer
            .log_frame(&frame2, Duration::from_millis(200), 1, false)
            .unwrap();
        writer
            .log_frame(&frame3, Duration::from_millis(300), 2, true)
            .unwrap();

        writer.close().unwrap();

        // Verify all frames were logged
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("0.100000"));
        assert!(content.contains("0.200000"));
        assert!(content.contains("0.300000"));
        assert!(content.contains("256")); // 0x100
        assert!(content.contains("512")); // 0x200
        assert!(content.contains("768x")); // 0x300
    }

    #[test]
    fn test_timestamp_precision() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut writer = AscWriter::create(path).unwrap();
        let frame = CanFrame::new_standard(0x123, &[0x00]).unwrap();

        // Test microsecond precision
        writer
            .log_frame(&frame, Duration::from_micros(1234567), 1, true)
            .unwrap();
        writer.close().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("1.234567"));
    }

    #[test]
    fn test_flush() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut writer = AscWriter::create(path).unwrap();
        let frame = CanFrame::new_standard(0x123, &[0x00]).unwrap();

        writer
            .log_frame(&frame, Duration::from_millis(100), 1, true)
            .unwrap();
        writer.flush().unwrap();

        // Verify data was flushed
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("0.100000"));
    }

    #[test]
    fn test_header_format() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let writer = AscWriter::create(path).unwrap();
        drop(writer);

        let content = fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Verify header structure
        assert!(lines.len() >= 4);
        assert!(lines[0].starts_with("date"));
        assert_eq!(lines[1], "base hex  timestamps absolute");
        assert_eq!(lines[2], "no internal events logged");
        assert_eq!(lines[3], "// version 7.1.0");
    }

    #[test]
    fn test_empty_frame() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut writer = AscWriter::create(path).unwrap();
        let frame = CanFrame::new_standard(0x123, &[]).unwrap();

        writer
            .log_frame(&frame, Duration::from_millis(100), 1, true)
            .unwrap();
        writer.close().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("d 0"));
    }

    #[test]
    fn test_max_dlc_frame() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut writer = AscWriter::create(path).unwrap();
        let frame = CanFrame::new_standard(0x123, &[0, 1, 2, 3, 4, 5, 6, 7]).unwrap();

        writer
            .log_frame(&frame, Duration::from_millis(100), 1, true)
            .unwrap();
        writer.close().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("d 8"));
        assert!(content.contains("00 01 02 03 04 05 06 07"));
    }
}
