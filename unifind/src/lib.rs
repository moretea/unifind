#[derive(Debug)]
pub struct UnicodeDefinition {
    codepoint: u32,
    name: &'static str,
    general_category: UnicodeGeneralCategory,
}

// See http://www.unicode.org/reports/tr44/#General_Category_Values
#[derive(Debug)]
enum UnicodeGeneralCategory {
    LC,
    Lu,
    Ll,
    Lt,
    L,
    Lm,
    Lo,
    M,
    Mn,
    Mc,
    Me,
    N,
    Nd,
    Nl,
    No,
    P,
    Pc,
    Pd,
    Ps,
    Pe,
    Pi,
    Pf,
    Po,
    S,
    Sm,
    Sc,
    Sk,
    So,
    Z,
    Zs,
    Zl,
    Zp,
    C,
    Cc,
    Cf,
    Cs,
    Co,
    Cn,
}

impl UnicodeGeneralCategory {
    fn parse(category: &str) -> Result<Self, &str> {
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

        def_cats!(LC,
                  Lu,
                  Ll,
                  Lt,
                  L,
                  Lm,
                  Lo,
                  M,
                  Mn,
                  Mc,
                  Me,
                  N,
                  Nd,
                  Nl,
                  No,
                  P,
                  Pc,
                  Pd,
                  Ps,
                  Pe,
                  Pi,
                  Pf,
                  Po,
                  S,
                  Sm,
                  Sc,
                  Sk,
                  So,
                  Z,
                  Zs,
                  Zl,
                  Zp,
                  C,
                  Cc,
                  Cf,
                  Cs,
                  Co,
                  Cn)
    }
}

mod trie {
    use super::UnicodeDefinition;
    use super::UnicodeGeneralCategory;

    struct Trie {
        definitions: &'static [u32],
        children: &'static [(char, u32)],
    }

    include!(concat!("generated.rs"));

    pub fn find(term: &str, mut results: &mut Vec<u32>) {
        find_in_trie(&TRIE_DEFS[0], term.chars(), &mut results)
    }

    fn find_in_trie(trie: &Trie, mut chars: ::std::str::Chars, mut results: &mut Vec<u32>) {
        if let Some(chr) = chars.next() {
            // drill down
            if let Ok(pos) = trie.children.binary_search_by(|&(key, idx)| key.cmp(&chr)) {
                let &(key, trie_pos) = trie.children.get(pos).unwrap();
                let trie = &TRIE_DEFS[trie_pos as usize];
                find_in_trie(&trie, chars, &mut results)
            } else {
                // found nothing
                return;
            }
        } else {
            // get all stuff below (and including) this trie
            collect_into(&trie, &mut results);
        }
    }

    fn collect_into(trie: &Trie, mut into: &mut Vec<u32>) {
        for def in trie.definitions {
            into.push(*def);
        }

        for &(key, idx) in trie.children {
            let trie = &TRIE_DEFS[idx as usize];
            collect_into(&trie, &mut into);
        }
    }
}

#[derive(Debug)]
pub struct SearchResult {
    def_idx: u32,
    // definition: &'static UnicodeDefinition,
    hits: u32,
}

impl SearchResult {
    fn definition(&self) -> &'static UnicodeDefinition {
        &trie::UNICODE_DEFS[self.def_idx as usize]
    }
}

impl std::fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let def = self.definition();
        let chr = std::char::from_u32(def.codepoint)
                      .unwrap_or_else(|| panic!("Illegal unicode character!"));
        write!(f, "{:X}\t{}\t{}", def.codepoint, chr, def.name)
    }
}

pub fn search_or<T: AsRef<str>>(terms: &[T]) -> Vec<SearchResult> {
    let mut raw_results = Vec::new();

    for term in terms {
        trie::find(term.as_ref(), &mut raw_results);
    }

    let mut results: Vec<SearchResult> = Vec::new();
    for raw_result in raw_results {
        let pos = results.iter().position(|ref result| result.def_idx == raw_result);

        if let Some(pos) = pos {
            results[pos].hits += 1;
        } else {
            results.push(SearchResult {
                def_idx: raw_result,
                hits: 1,
            });
        }
    }

    results.sort_by(|a, b| a.hits.cmp(&b.hits));
    results
}

pub fn search_and<T: AsRef<str>>(terms: &[T]) -> Vec<SearchResult> {
    let mut raw_results = Vec::new();

    for term in terms {
        trie::find(term.as_ref(), &mut raw_results);
    }

    let mut results: Vec<SearchResult> = Vec::new();
    for raw_result in raw_results {
        let pos = results.iter().position(|ref result| result.def_idx == raw_result);

        if let Some(pos) = pos {
            results[pos].hits += 1;
        } else {
            results.push(SearchResult {
                def_idx: raw_result,
                hits: 1,
            });
        }
    }

    results.retain(|ref result| result.hits >= terms.len() as u32);
    results
}
