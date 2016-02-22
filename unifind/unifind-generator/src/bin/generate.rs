use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::path::Path;
use std::collections::HashMap;

fn main() {
    let target_path = Path::new("../src/generated.rs");
    let target = File::create(&target_path).expect("Could not create trie.rs!");

    let source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let source_path = Path::new(&source_dir).join("UnicodeData.txt");
    let source = File::open(source_path).expect("Could not open UnicodeData.txt");

    let (trie, defs) = build_trie(source);
    let trie = add_idx_to_trie(trie);
    let flat_trie = trie.flatten();

    generate_code(flat_trie, defs, target);
}

fn generate_code(flat_trie: Vec<FlatTrie>, defs: Vec<UnicodeDefinition>, mut target: File) {
    target.write_all(b"pub static UNICODE_DEFS: &'static [UnicodeDefinition] = &[\n").unwrap();
    for def in defs {
        write!(target,"  UnicodeDefinition {{ name: \"{}\", codepoint: {}, general_category: UnicodeGeneralCategory::{:?} }},\n", def.name, def.codepoint, def.general_category).unwrap();
    }
    target.write_all(b"];\n").unwrap();

    target.write_all(b"\n").unwrap();

    target.write_all(b"static TRIE_DEFS: &'static [Trie] = &[\n").unwrap();
    for t in flat_trie {
        write!(target,"  Trie {{ definitions: &{:?}, children: &{:?} }},\n", t.definitions, t.children).unwrap();
    }

    target.write_all(b"];\n").unwrap();
}

fn build_trie(source: File) -> (Trie, Vec<UnicodeDefinition>) {
    let mut trie = Trie::new();
    let mut definitions = Vec::new();

    let source = BufReader::new(source);

    let mut definition_idx : DefinitionIndex = 0;

    for line in source.lines() {
        let line = line.unwrap_or_else(|e| panic!("Illegal line: {:?}", e));
        let mut columns = line.split(";");
        let codepoint_str   = columns.next().expect("illegal format");
        let name        = columns.next().expect("illegal format");
        let category_str    = columns.next().expect("illegal format");

        let codepoint = u32::from_str_radix(&codepoint_str, 16).unwrap_or_else(|e| panic!("Could not parse codepoint hex number: {:?}", e));
        let category = UnicodeGeneralCategory::parse(category_str).unwrap_or_else(|e| panic!("Could not parse category {:?}", e));

        let def = UnicodeDefinition {
            codepoint: codepoint,
            name: name.into(),
            general_category: category
        };

        for word in def.name.split_whitespace() {
            let name_chars : Vec<char> = word.chars().collect();
            trie.add(&name_chars, definition_idx);
        }

        definition_idx += 1;
        definitions.push(def);
    }

    (trie, definitions)
}

#[derive(Debug)]
struct Trie {
    idx: i32,
    children: HashMap<char, Trie>,
    definitions: Vec<DefinitionIndex>,
}

struct FlatTrie {
    idx: i32,
    definitions: Vec<i32>,
    children: Vec<(char, i32)>
}

type TrieContext = i32;

impl Trie {
    pub fn new() -> Trie {
        Trie {
            idx: 0,
            children: HashMap::new(),
            definitions: Vec::new(),
        }
    }

    fn add(&mut self, key_chars: &[char], def_index: DefinitionIndex) {
        if let Some((chr, rest)) = key_chars.split_first() {
            if !self.children.contains_key(&chr) {
                self.children.insert(*chr, Trie::new());
            }

            let child = self.children.get_mut(&chr).unwrap();

            child.add(rest, def_index);
        } else {
            self.definitions.push(def_index);
        }
    }

    fn size(&self) -> i32 {
        self.children.values().fold(0, |acc, ref trie|  acc + trie.size()) + 1
    }

    fn flatten(self) -> Vec<FlatTrie> {
        let mut result = Vec::with_capacity(self.size() as usize);

        fn do_flat(mut t: Trie, mut target: &mut Vec<FlatTrie>) {
            let mut children = Vec::new();

            for (key, child) in t.children.drain() {
                children.push((key, child.idx));
                do_flat(child, &mut target);
            }

            children.sort_by(|&(a_key, _a_def_idx), &(b_key, _b_def_idx)| a_key.cmp(&b_key));

            target.push(FlatTrie {
                idx: t.idx,
                definitions: t.definitions,
                children: children
            });
        }

        do_flat(self, &mut result);

        result.sort_by(|a,b| a.idx.cmp(&b.idx));

        result
    }
}

fn add_idx_to_trie(mut t: Trie) -> Trie {
    fn add_number_to_children(children: &mut HashMap<char, Trie>, mut idx: i32) -> i32 {
        for (_k, child) in children.iter_mut() {
            child.idx = idx;
            idx = add_number_to_children(&mut child.children, idx + 1);
        }
        return idx;
    }

    t.idx = 0;
    add_number_to_children(&mut t.children, 1);
    t
}

type DefinitionIndex = i32;

// See http://www.unicode.org/reports/tr44/#UnicodeData.txt
#[derive(Debug)]
struct UnicodeDefinition {
    codepoint: u32,
    name: String,
    general_category: UnicodeGeneralCategory
}

// See http://www.unicode.org/reports/tr44/#General_Category_Values
#[derive(Debug)]
enum UnicodeGeneralCategory {
    LC, Lu, Ll, Lt,
    L, Lm, Lo,
    M, Mn, Mc, Me,
    N, Nd, Nl, No,
    P, Pc, Pd, Ps, Pe, Pi, Pf, Po,
    S, Sm, Sc, Sk, So,
    Z, Zs, Zl, Zp,
    C, Cc, Cf, Cs, Co, Cn,
}

impl UnicodeGeneralCategory {
    fn parse(category: &str) -> Result<Self,&str> {
        macro_rules! def_cats {
            ( $($cat: ident), *) => {
                match category {
                    $(
                        stringify!($cat) => { Ok(UnicodeGeneralCategory::$cat) },
                    )*
                    _ => { Err(category) }
                }
            }
        }

        def_cats!(
            LC, Lu, Ll, Lt,
            L, Lm, Lo,
            M, Mn, Mc, Me,
            N, Nd, Nl, No,
            P, Pc, Pd, Ps, Pe, Pi, Pf, Po,
            S, Sm, Sc, Sk, So,
            Z, Zs, Zl, Zp,
            C, Cc, Cf, Cs, Co, Cn
        )
    }
}
