//! Save and load programs and associated data from files.
//!
//! # File format
//!
//! All values are stored in big endian byte order.
//! The header occupies 16 bytes followed by a flat instruction stream:
//!
//! | Offset | Size    | Field           | Description                                                                 |
//! |--------|---------|-----------------|-----------------------------------------------------------------------------|
//! | 0      | 4 bytes | Magic           | Always `0x001A_D1BC` as `u32`.                                              |
//! | 4      | 4 bytes | Version         | Contains crate version as top three bytes, format version as last byte.     |
//! | 8      | 4 bytes | Flags           | [`FileFlags`] feature bits encoded as `u32`.                                |
//! | 12     | 4 bytes | Data word count | Number of input [`MachineWord`]s following the header.                      |
//!
//! After the header the remainder of the file is a flat sequence of [`Instruction`]-sized values,
//! each occupying [`INSTRUCTION_SIZE`] bytes.  The first `data_word_count * INSTR_PER_WORD`
//! values encode the interpreter input data; the rest is the program bytecode.
//!
//! Loading a file whose feature flags (excluding [`FileFlags::DATA_ATTACHED`]) differ from those
//! expected by the running build is rejected.  A non-zero data word count requires
//! [`FileFlags::DATA_ATTACHED`] to be set.

use std::io::{Read, Write};

use bitflags::bitflags;

use crate::consts::{INSTR_PER_WORD, INSTRUCTION_SIZE, Instruction, MachineWord};
use crate::util::{instructions_from_word, word_from_instructions};

const MAGIC: u32 = 0x001A_D1BC;

bitflags! {
    /// Reprensents the set of flags that could be set in a file.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FileFlags: u32 {
        /// `int-word` crate feature enabled?
        const INT_WORD        = 0x0000_0001;
        /// `long-word` crate feature enabled?
        const LONG_WORD       = 0x0000_0002;
        /// `long-instruction` crate feature enabled?
        const LONG_INSTRUCTION = 0x0000_0004;
        /// `long-address` crate feature enabled?
        const LONG_ADDRESS    = 0x0000_0008;
        /// Is there data attached or just instructions
        const DATA_ATTACHED   = 0x0000_0010;
    }
}

const fn expected_flags() -> FileFlags {
    let mut flags = FileFlags::empty();
    if cfg!(feature = "int-word") {
        flags = flags.union(FileFlags::INT_WORD);
    }
    if cfg!(feature = "long-word") {
        flags = flags.union(FileFlags::LONG_WORD);
    }
    if cfg!(feature = "long-instruction") {
        flags = flags.union(FileFlags::LONG_INSTRUCTION);
    }
    if cfg!(feature = "long-address") {
        flags = flags.union(FileFlags::LONG_ADDRESS);
    }
    flags
}

const EXPECTED: FileFlags = expected_flags();

// Converts decimal string representation of number to [`u8`].
// This will truncate to `u8` if the number is larger than that.
#[allow(clippy::cast_possible_truncation)]
const fn parse_version_to_byte(s: &str) -> u8 {
    let str_bytes = s.as_bytes();
    let mut rslt: u32 = 0;
    let mut i: usize = 0;
    while i < str_bytes.len() {
        let ch = str_bytes[i];
        assert!(ch >= b'0' && ch <= b'9', "non-digit in version component");
        rslt *= 10;
        rslt += (ch - b'0') as u32;
        i += 1;
    }
    rslt as u8
}

const MAJOR: u32 = parse_version_to_byte(env!("CARGO_PKG_VERSION_MAJOR")) as u32;
const MINOR: u32 = parse_version_to_byte(env!("CARGO_PKG_VERSION_MINOR")) as u32;
const PATCH: u32 = parse_version_to_byte(env!("CARGO_PKG_VERSION_PATCH")) as u32;
const FORMAT_VERSION: u32 = 11;

const VERSION_U32: u32 = (MAJOR << 24) | (MINOR << 16) | (PATCH << 8) | FORMAT_VERSION;

/// Errors that can occur when reading or writing a tardi file.
#[derive(Debug)]
pub enum FileError {
    /// An underlying IO error occurred.
    Io(std::io::Error),
    /// The file does not begin with the expected magic number.
    InvalidMagic,
    /// The feature flags stored in the file do not match the current build.
    FlagMismatch,
    /// The format version supplied in the file does not march `FORMAT_VERSION`.
    VersionMissmatch,
    /// The file is malformed (truncated, misaligned, or has an inconsistent data word count).
    InvalidFormat,
}

impl From<std::io::Error> for FileError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::InvalidMagic => f.write_str("invalid magic number"),
            Self::FlagMismatch => f.write_str("feature flags do not match the current build"),
            Self::VersionMissmatch => f.write_str("incompatible format version"),
            Self::InvalidFormat => f.write_str("malformed file format"),
        }
    }
}

impl std::error::Error for FileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// Write `instructions` and `data` words to `writer` in the tardi file format.
///
/// # Errors
///
/// - [`FileError::InvalidFormat`] if `data` contains more than `u32::MAX` words.
/// - [`FileError::Io`] if any write to `writer` fails.
pub fn write_program<W: Write>(
    mut writer: W,
    instructions: &[Instruction],
    data: &[MachineWord],
) -> Result<(), FileError> {
    writer.write_all(&MAGIC.to_be_bytes())?;

    writer.write_all(&VERSION_U32.to_be_bytes())?;
    
    let mut flags = EXPECTED;
    if !data.is_empty() {
        flags |= FileFlags::DATA_ATTACHED;
    }
    writer.write_all(&flags.bits().to_be_bytes())?;

    let data_count = u32::try_from(data.len()).map_err(|_| FileError::InvalidFormat)?;
    writer.write_all(&data_count.to_be_bytes())?;

    for word in data {
        let parts = instructions_from_word(*word);
        for instr in &parts {
            writer.write_all(&instr.to_be_bytes())?;
        }
    }

    for instr in instructions {
        writer.write_all(&instr.to_be_bytes())?;
    }

    Ok(())
}

/// Read a tardi file from `reader` and return `(program, input)` ready to pass to an [`Interpreter`](`crate::interpreter::Interpreter`).
///
/// # Errors
///
/// - [`FileError::InvalidMagic`] if the file does not begin with the expected magic number.
/// - [`FileError::FlagMismatch`] if the feature flags in the file differ from the current build.
/// - [`FileError::VersionMissmatch`] if the format version byte does not equal `FORMAT_VERSION`.
/// - [`FileError::InvalidFormat`] if the file is truncated, the instruction stream is misaligned,
///   or a non-zero data word count appears without [`FileFlags::DATA_ATTACHED`] being set.
/// - [`FileError::Io`] if any read from `reader` fails.
pub fn read_program<R: Read>(
    mut reader: R,
) -> Result<(Vec<Instruction>, Vec<MachineWord>), FileError> {
    let mut buf4 = [0_u8; 4];

    reader.read_exact(&mut buf4)?;
    if u32::from_be_bytes(buf4) != MAGIC {
        return Err(FileError::InvalidMagic);
    }

    reader.read_exact(&mut buf4)?;
    let file_format_version = u32::from_be_bytes(buf4) & 0xFF;
    if file_format_version != FORMAT_VERSION {
        return Err(FileError::VersionMissmatch);
    }

    reader.read_exact(&mut buf4)?;
    let file_flags = FileFlags::from_bits_truncate(u32::from_be_bytes(buf4));
    if file_flags.difference(FileFlags::DATA_ATTACHED) != EXPECTED {
        return Err(FileError::FlagMismatch);
    }

    reader.read_exact(&mut buf4)?;
    let data_word_count = u32::from_be_bytes(buf4) as usize;
    if data_word_count > 0 && !file_flags.contains(FileFlags::DATA_ATTACHED) {
        return Err(FileError::InvalidFormat);
    }

    let mut raw_bytes: Vec<u8> = Vec::new();
    let _bytes_read = reader.read_to_end(&mut raw_bytes)?;

    #[allow(clippy::modulo_one)]
    if raw_bytes.len() % INSTRUCTION_SIZE != 0 {
        return Err(FileError::InvalidFormat);
    }

    let data_instr_count = data_word_count * INSTR_PER_WORD;
    let total_instr_count = raw_bytes.len() / INSTRUCTION_SIZE;
    if total_instr_count < data_instr_count {
        return Err(FileError::InvalidFormat);
    }

    let mut all_instrs: Vec<Instruction> = Vec::with_capacity(total_instr_count);
    for chunk in raw_bytes.chunks_exact(INSTRUCTION_SIZE) {
        let arr: [u8; INSTRUCTION_SIZE] = chunk.try_into().map_err(|_| FileError::InvalidFormat)?;
        all_instrs.push(Instruction::from_be_bytes(arr));
    }

    let mut input: Vec<MachineWord> = Vec::with_capacity(data_word_count);
    for chunk in all_instrs[..data_instr_count].chunks_exact(INSTR_PER_WORD) {
        let mut arr = [Instruction::default(); INSTR_PER_WORD];
        arr.copy_from_slice(chunk);
        input.push(word_from_instructions(arr));
    }

    let program = all_instrs[data_instr_count..].to_vec();
    Ok((program, input))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    use crate::{
        instr::OpCode,
        mw,
        util::{patch_jmp, push_jmp},
    };

    #[test]
    fn round_trip_to_buffer() {
        let mut original_program: Vec<Instruction> =
            vec![OpCode::Add.into(), OpCode::PushImm.into()];
        original_program.extend_from_slice(&instructions_from_word(mw!(123)));
        let offset = push_jmp(&mut original_program, OpCode::Jmp);
        patch_jmp(&mut original_program, offset, 0);
        let original_data = vec![mw!(12345), mw!(67890), mw!(0), mw!(0)];

        let mut buf = Vec::new();
        write_program(&mut buf, &original_program, &original_data)
            .expect("encoding to buffer should throw no errors");
        let mut cursor = Cursor::new(buf);
        let (decoded_program, decoded_data) =
            read_program(&mut cursor).expect("decoding known good buffer should thrown no error");
        assert_eq!(
            original_program, decoded_program,
            "program recovered from buffer should match original"
        );
        assert_eq!(
            original_data, decoded_data,
            "data recovered from buffer should match original"
        );
    }
}
