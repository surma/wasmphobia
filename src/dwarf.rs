use std::collections::HashMap;

use fallible_iterator::FallibleIterator;
use gimli::{
    read::AttributeValue, DebuggingInformationEntry, EndianSlice, LittleEndian, Reader, Unit,
};

macro_rules! unwrap_or_continue {
    ($v:expr) => {
        match $v {
            Some(v) => v,
            _ => continue,
        }
    };
}

macro_rules! unwrap_or_ok_none {
    ($v:expr) => {
        match $v {
            Some(v) => v,
            _ => return Ok(None),
        }
    };
}

fn unpack_size<R: gimli::Reader>(low: &AttributeValue<R>, high: &AttributeValue<R>) -> Option<u64> {
    let AttributeValue::Addr(low) = *low else {
        return None;
    };
    match high {
        AttributeValue::Addr(v) => Some(*v - low),
        AttributeValue::Udata(v) => Some(*v),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
pub struct DwarfAnalysisOpts {
    pub prefix: Option<String>,
    pub compilation_units: bool,
    pub split_paths: bool,
}

pub fn analyze_dwarf(
    dwarf: gimli::Dwarf<EndianSlice<'_, LittleEndian>>,
    opts: &DwarfAnalysisOpts,
) -> anyhow::Result<HashMap<String, u64>> {
    let mut contributors: HashMap<String, u64> = HashMap::new();
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;
        let unit_name = unit
            .name
            .and_then(|s| s.to_string().ok())
            .unwrap_or("<unknown compilation unit>")
            .trim_start_matches('/');

        let mut entries = unit.entries();
        while let Some((_, entry)) = entries.next_dfs()? {
            if !matches!(
                entry.tag(),
                gimli::DW_TAG_subprogram | gimli::DW_TAG_inlined_subroutine
            ) {
                continue;
            }
            let mut key = vec![];
            if let Some(prefix) = &opts.prefix {
                key.push(prefix.to_string());
            }
            if opts.compilation_units {
                key.push(format!("@compilation_unit: {unit_name}"))
            }
            let (dir, file, _entry_name, size) =
                unwrap_or_continue!(process_die(entry, &unit, &dwarf)?);
            if opts.split_paths {
                key.push("@source_files".into());
                key.extend(dir.split('/').map(Into::into));
                key.push(file);
            } else {
                key.push(format!("@source_file: {dir}/{file}"));
            };
            key.push("@function: entry_name".to_string());
            let key = key.join(";");
            *contributors.entry(key).or_insert(0) += size;
        }
    }
    Ok(contributors)
}

fn process_die<R: gimli::Reader>(
    entry: &DebuggingInformationEntry<'_, '_, R>,
    unit: &Unit<R>,
    dwarf: &gimli::Dwarf<R>,
) -> anyhow::Result<Option<(String, String, String, u64)>> {
    let size = unwrap_or_ok_none!(entry_mapped_size(entry, unit, dwarf)?);

    let (dir, file) = unpack_file(entry, unit, dwarf)?
        .unwrap_or(("<unknown dir>".into(), "<unknown file>".into()));

    let entry_name = unwrap_or_ok_none!(entry.attr_value(gimli::DW_AT_name)?);
    let entry_name = unwrap_or_ok_none!(entry_name.string_value(&dwarf.debug_str));
    let entry_name = entry_name.to_string()?;

    let dir = if !dir.starts_with('/') && !dir.starts_with('<') {
        let unit_dir = unit.comp_dir.as_ref().and_then(|c| c.to_string().ok());
        unit_dir.unwrap_or("".into()).to_string() + &dir
    } else {
        dir.to_string()
    };

    Ok(Some((
        dir.to_string(),
        file.to_string(),
        entry_name.to_string(),
        size,
    )))
}

// If a DWARF Debugging Information Entry (DIE) references output code,
// it can fall into one of three scenarios:
// - It contains just a `low_pc` to reference a location (in memory or otherwise)
// - It contains `low_pc` and `high_pc` to reference a region
// - It contains a `ranges` attribue to reference multiple regions
//
// This function ignores the first case, and sums up the total bytes references
// by the other cases.
fn entry_mapped_size<R: gimli::Reader>(
    entry: &DebuggingInformationEntry<'_, '_, R>,
    unit: &Unit<R>,
    dwarf: &gimli::Dwarf<R>,
) -> anyhow::Result<Option<u64>> {
    // Deal with ranges first, as compilation units can have a low_pc _and_ a ranges attribute.
    if let Some(ranges) = entry.attr_value(gimli::DW_AT_ranges)? {
        let AttributeValue::RangeListsRef(list_ref) = ranges else {
            return Ok(None);
        };
        let range_list_offset = dwarf.ranges_offset_from_raw(unit, list_ref);
        let ranges = dwarf.ranges(unit, range_list_offset)?;
        let sum = ranges
            .map(|range| Ok(range.end - range.begin))
            .fold(0, |acc, d| Ok(acc + d))?;
        return Ok(Some(sum));
    };
    let low_pc = unwrap_or_ok_none!(entry.attr_value(gimli::DW_AT_low_pc)?);
    let high_pc = unwrap_or_ok_none!(entry.attr_value(gimli::DW_AT_high_pc)?);
    Ok(unpack_size(&low_pc, &high_pc))
}

fn unpack_file<R: Reader>(
    entry: &DebuggingInformationEntry<'_, '_, R>,
    unit: &gimli::Unit<R>,
    dwarf: &gimli::Dwarf<R>,
) -> anyhow::Result<Option<(String, String)>> {
    if let Some(AttributeValue::UnitRef(r)) = entry.attr_value(gimli::DW_AT_abstract_origin)? {
        let entry = unit.entry(r)?;
        unpack_file(&entry, unit, dwarf)
    } else if let Some(AttributeValue::FileIndex(file_index)) =
        entry.attr_value(gimli::DW_AT_decl_file)?
    {
        let header = unwrap_or_ok_none!(unit.line_program.as_ref()).header();
        let file = unwrap_or_ok_none!(header.file(file_index));
        let dir = unwrap_or_ok_none!(file.directory(header));
        let dir = unwrap_or_ok_none!(dir.string_value(&dwarf.debug_str));
        let dir = unwrap_or_ok_none!(dir.to_string().ok());
        let file_name = file.path_name();
        let file_name = unwrap_or_ok_none!(file_name.string_value(&dwarf.debug_str));
        let file_name = unwrap_or_ok_none!(file_name.to_string().ok());
        Ok(Some((dir.to_string(), file_name.to_string())))
    } else {
        Ok(None)
    }
}
