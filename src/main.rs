use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use gimli::{EndianSlice, LittleEndian};

#[derive(Debug, Parser)]
pub struct Args {
    input: PathBuf,
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let content = std::fs::read(&args.input)?;
    let module = walrus::Module::from_buffer(&content)?;
    let dwarf = module.debug.dwarf;
    let dwarf = dwarf.borrow(|v| EndianSlice::new(v.as_slice(), LittleEndian));

    let mut contributors = accumulate_contributors(dwarf)?;
    let data_sections = module.data.iter().enumerate().map(|(idx, data)| {
        let name = format!(
            "/@wasm_binary/data_sections/{}",
            data.name.clone().unwrap_or_else(|| idx.to_string())
        );
        (name, u64::try_from(data.value.len()).unwrap())
    });
    contributors.extend(data_sections);
    let keys: Vec<_> = contributors
        .keys()
        .map(|s| s.split('/').collect::<Vec<_>>())
        .collect();
    let tree = PrefixTreeNode::build(keys);

    let mut output = std::fs::File::create(args.output)?;

    let inferno_lines = to_inferno_lines(tree, &contributors);
    let mut options = inferno::flamegraph::Options::default();
    options.title = args
        .input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("<Unknown wasm file>")
        .to_string();
    options.subtitle = Some("Contribution to the code section per compilation unit".to_string());
    options.count_name = "KB".to_string();
    options.notes = "The flamegraph lololol".to_string();
    options.factor = 1.0 / 1000.0;
    inferno::flamegraph::from_lines(
        &mut options,
        inferno_lines.iter().map(|v| v.as_str()),
        &mut output,
    )?;

    Ok(())
}

fn to_inferno_lines(tree: PrefixTreeNode<'_>, contributors: &HashMap<String, u64>) -> Vec<String> {
    let inferno_lines: Vec<_> = tree
        .dfs()
        .into_iter()
        .map(|entry| {
            let size = if let Some(subtree) = tree.lookup(&entry) {
                let key = entry.join("/");
                total_size(contributors, key, subtree)
            } else {
                0
            };
            (entry, size)
        })
        .map(|(entry, size)| format!("{} {}", entry.join(";"), size))
        .collect();
    inferno_lines
}

fn total_size(contributors: &HashMap<String, u64>, key: String, _tree: &PrefixTreeNode<'_>) -> u64 {
    let size = contributors.get(&key).copied().unwrap_or(0);
    size
}

fn accumulate_contributors(
    dwarf: gimli::Dwarf<EndianSlice<'_, LittleEndian>>,
) -> Result<HashMap<String, u64>, anyhow::Error> {
    let mut contributors: HashMap<String, u64> = HashMap::new();
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
    Ok(contributors)
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

    fn lookup(&self, prefix: &[&'a str]) -> Option<&PrefixTreeNode> {
        let len = prefix.len().min(self.prefix.len());
        if self.prefix[0..len] != prefix[0..len] {
            return None;
        }
        if len == self.prefix.len() {
            return Some(self);
        }
        let new_prefix = &prefix[self.prefix.len()..];
        self.children
            .iter()
            .find_map(|child| child.lookup(new_prefix))
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
