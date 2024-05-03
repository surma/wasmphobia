use std::collections::HashMap;

use anyhow::anyhow;
use fallible_iterator::FallibleIterator;
use gimli::{
    read::AttributeValue, DebuggingInformationEntry, EndianSlice, EntriesCursor, LittleEndian,
    Reader, Unit,
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

type Contributors = HashMap<String, u64>;

#[derive(Debug, Clone)]
pub struct DwarfAnalyzer<'a, R: Reader> {
    pub prefix: Option<String>,
    pub compilation_units: bool,
    pub split_paths: bool,
    pub dwarf: &'a mut gimli::Dwarf<R>,
    // pub unit_cursor:
    pub contributors: Contributors,
    // pub unit_cursor:
}

impl<'a, R: Reader> DwarfAnalyzer<'a, R> {
    fn analyze(self) -> anyhow::Result<Contributors> {
        let mut iter = self.dwarf.units();
        while let Some(header) = iter.next()? {
            let unit = self.dwarf.unit(header)?;
            let unit_name = unit
                .name
                .and_then(|s| s.to_string().ok())
                .unwrap_or("<unknown compilation unit>");

            let mut entry_cursor = unit.entries();
            entry_cursor
                .next_entry()?
                .ok_or_else(|| anyhow!("DWARF data has no entries"))?;
            loop {
                self.analyze_die_subtree(entry_cursor.clone(), &unit)?;
                if entry_cursor.next_sibling()?.is_non() {
                    break;
                }
            }
        }
        Ok(self.contributors)
    }

    fn analyze_die_subtree(
        &mut self,
        mut entry_cursor: gimli::EntriesCursor<'_, '_, R>,
        unit: &gimli::Unit<R>,
    ) -> anyhow::Result<()> {
        let entry = entry_cursor
            .current()
            .ok_or_else(|| anyhow!("Analysis was started on an empty tree"))?;

        if !matches!(
            entry.tag(),
            gimli::DW_TAG_subprogram | gimli::DW_TAG_inlined_subroutine
        ) {
            return Ok(());
        }

        let (dir, file, name, mut size) = self.analyze_die(entry, unit)?.ok_or_else(|| {
            anyhow!("DWARF entry {entry} is a subprogram or inlined subroutine, but has no mapping data")
        })?;

        if entry.has_children() {
            entry_cursor
                .next_entry()?
                .expect("Guaranteed by has_children");
            loop {
                self.analyze_die_subtree(entry_cursor.clone(), unit)?;
                if entry_cursor.next_sibling()?.is_none() {
                    break;
                }
            }
            // let total_children_size: u64 = result.values().sum();
            // size = size.checked_sub(total_children_size).ok_or_else(|| {
            //     anyhow!(
            //     "Children of {name} from {dir}/{file} add up to more bytes than the item itself"
            // )
            // })?;
        }

        let mut key = vec![];
        key.extend(dir.split('/').map(Into::into));
        key.push(file);
        key.push(format!("@function: {name}"));
        let key = key.join(";");
        self.contributors.entry(key).or_insert(0) += size;
        Ok(())
    }

    fn analyze_die(
        &mut self,
        entry: &DebuggingInformationEntry<'_, '_, R>,
        unit: &Unit<R>,
    ) -> anyhow::Result<Option<(String, String, String, u64)>> {
        let size = unwrap_or_ok_none!(self.entry_mapped_size(entry, unit)?);

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
}
