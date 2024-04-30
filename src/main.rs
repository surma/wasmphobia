use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use gimli::{EndianSlice, LittleEndian};

#[derive(Debug, Parser)]
pub struct Args {
    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let content = std::fs::read(args.input)?;
    let module = walrus::Module::from_buffer(&content)?;
    let dwarf = module.debug.dwarf;
    let dwarf = dwarf.borrow(|v| EndianSlice::new(v.as_slice(), LittleEndian));

    let mut contributors: HashMap<String, u64> = HashMap::new();
    let data_sections = module.data.iter().enumerate().map(|(idx, data)| {
        let name = format!(
            "/data/{}",
            data.name.clone().unwrap_or_else(|| idx.to_string())
        );
        (name, u64::try_from(data.value.len()).unwrap())
    });
    contributors.extend(data_sections);
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;
        let name = unit
            .name
            .map(|s| std::str::from_utf8(s.slice()).unwrap_or("<Invalid utf8>"))
            .unwrap_or("<Unknown>")
            .to_string();

        let mut size = 0;
        let mut ranges = dwarf.unit_ranges(&unit)?;
        while let Some(range) = ranges.next()? {
            size += range.end - range.begin;
        }

        *contributors.entry(name).or_insert(0) += size;
    }
    println!("{contributors:#?}");
    let keys: Vec<_> = contributors
        .keys()
        .map(|s| s.split('/').collect::<Vec<_>>())
        .collect();
    let tree = PrefixTreeNode::build(keys);
    println!("{:#?}", tree.dfs());

    Ok(())
}

#[derive(Debug, Clone)]
struct PrefixTreeNode<'a> {
    prefix: Vec<&'a str>,
    children: Vec<PrefixTreeNode<'a>>,
}

impl<'a> PrefixTreeNode<'a> {
    fn build(mut data: Vec<Vec<&'a str>>) -> PrefixTreeNode<'a> {
        let prefix = longest_common_prefix(&data).to_vec();
        data.iter_mut().for_each(|item| {
            item.copy_within(prefix.len().., 0);
            item.truncate(item.len() - prefix.len());
        });
        data.retain(|l| !l.is_empty());
        PrefixTreeNode {
            prefix,
            children: PrefixTreeNode::build_children(data),
        }
    }

    // Invariant: data has no common prefix
    fn build_children(data: Vec<Vec<&'a str>>) -> Vec<PrefixTreeNode<'a>> {
        let mut groups: HashMap<&'a str, Vec<Vec<&'a str>>> = HashMap::new();
        for item in data {
            groups.entry(item[0]).or_default().push(item)
        }
        groups.into_values().map(PrefixTreeNode::build).collect()
    }

    fn dfs(&self) -> Vec<Vec<&'a str>> {
        let mut r = vec![self.prefix.clone()];
        for item in &self.children {
            r.extend(item.dfs().into_iter().map(|item| {
                let mut p = self.prefix.clone();
                p.extend(item);
                p
            }));
        }

        r
    }
}

fn longest_common_prefix<A: AsRef<[I]>, I: PartialEq>(s: &[A]) -> &'_ [I] {
    let mut prefix = &s[0].as_ref()[0..0];

    let max = s.iter().map(|s| s.as_ref().len()).min().unwrap_or(0);
    for i in 0..=max {
        let maybe_prefix = &s[0].as_ref()[0..i];
        if !s.iter().all(|seg| &seg.as_ref()[0..i] == maybe_prefix) {
            break;
        }
        prefix = maybe_prefix;
    }

    prefix
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn lcp() {
        assert_eq!(
            longest_common_prefix(&["hello world", "hello peter"]),
            "hello ".as_bytes()
        );
    }

    #[test]
    fn lcp_trivial() {
        assert_eq!(longest_common_prefix(&[&["test"]]), &["test"]);
    }

    #[test]
    fn tree() {
        let data = vec![
            "/Users/surma".split('/').collect::<Vec<_>>(),
            "/Users/surma/test".split('/').collect::<Vec<_>>(),
            "/Users/surma/Downloads".split('/').collect::<Vec<_>>(),
            "/tmp".split('/').collect::<Vec<_>>(),
        ];
        let tree = PrefixTreeNode::build(data);
        let items: HashSet<_> = tree
            .dfs()
            .into_iter()
            .map(|entry| entry.join("/"))
            .collect();

        let check_items = &["/Users/surma", "/tmp", "/Users/surma/test"];
        for item in check_items {
            assert!(items.contains(*item));
        }
        // assert!(items.c
    }
}
