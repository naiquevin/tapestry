use crate::error::{parse_error, Error};
use crate::query_template::QueryTemplates;
use crate::toml::{decode_pathbuf, decode_string, decode_vecstr};
use crate::validation::ManifestMistake;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml::Value;

#[allow(unused)]
#[derive(Debug)]
pub struct Query {
    id: String,
    template: PathBuf,
    conds: Vec<String>,
    output: Option<PathBuf>,
}

impl Query {
    fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        value: &Value,
    ) -> Result<Self, Error> {
        match value.as_table() {
            Some(t) => {
                let id = t
                    .get("id")
                    .ok_or(parse_error!("Missing 'id' in 'query' entry"))
                    .map(|v| decode_string(v, "queries[].id"))??;
                let template = t
                    .get("template")
                    .ok_or(parse_error!("Missing 'template' in 'query' entry"))
                    .map(|v| {
                        decode_pathbuf(v, Some(templates_base_dir.as_ref()), "queries[].template")
                    })??;
                let conds = t
                    .get("conds")
                    .ok_or(parse_error!("Missing 'conds' in 'query' entry"))
                    .map(|v| decode_vecstr(v, "queries[].conds"))??;
                let output = match t.get("option") {
                    Some(v) => Some(decode_pathbuf(
                        v,
                        Some(output_base_dir.as_ref()),
                        "queries[].output",
                    )?),
                    None => None,
                };
                Ok(Self {
                    id,
                    template,
                    conds,
                    output,
                })
            }
            None => Err(parse_error!("Invalid 'query' entry")),
        }
    }

    fn validate<'a, 'b>(&'a self, query_templates: &'b QueryTemplates) -> Vec<ManifestMistake<'a>>
    where
        'b: 'a,
    {
        let mut mistakes = vec![];
        match query_templates.get(&self.template) {
            Some(qt) => {
                // @TODO: Store conds (and all_conds) as HashSet instead of Vec
                let all_conds: HashSet<&str> = qt.all_conds.iter().map(|s| s.as_str()).collect();
                let conds: HashSet<&str> = self.conds.iter().map(|s| s.as_str()).collect();
                if !conds.is_subset(&all_conds) {
                    let diff = conds
                        .difference(&all_conds)
                        .copied()
                        .collect::<Vec<&str>>();
                    mistakes.push(ManifestMistake::InvalidConds {
                        query_id: &self.id,
                        conds: diff,
                    })
                }
            }
            None => mistakes.push(ManifestMistake::QueryTemplateRefNotFound {
                query_id: &self.id,
                template: self.template.to_str().unwrap(),
            }),
        }

        mistakes
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Queries {
    inner: Vec<Rc<Query>>,
    index: HashMap<String, Rc<Query>>,
}

impl Queries {
    pub fn new() -> Self {
        let inner: Vec<Rc<Query>> = vec![];
        let index: HashMap<String, Rc<Query>> = HashMap::new();
        Self { inner, index }
    }

    pub fn decode<P: AsRef<Path>>(
        templates_base_dir: P,
        output_base_dir: P,
        value: &Value,
    ) -> Result<Self, Error> {
        // @NOTE: The index is populated at the time of initialization
        // to avoid complexity. A lazy and memory efficient approach
        // would be populating the index at the time of lookup (like a
        // read-through cache) but in that case we'd need to manage
        // multiple mutable references.
        let mut index: HashMap<String, Rc<Query>> = HashMap::new();
        let items = match value.as_array() {
            Some(xs) => {
                let mut res = Vec::with_capacity(xs.len());
                for x in xs {
                    let q = Rc::new(Query::decode(&templates_base_dir, &output_base_dir, x)?);
                    let idx_key = q.id.clone();
                    let idx_val = q.clone();
                    res.push(q);
                    index.insert(idx_key, idx_val);
                }
                res
            }
            None => return Err(parse_error!("Invalid queries")),
        };
        Ok(Self {
            inner: items,
            index,
        })
    }

    pub fn validate<'a, 'b>(
        &'a self,
        query_templates: &'b QueryTemplates,
    ) -> Vec<ManifestMistake<'a>>
    where
        'b: 'a,
    {
        let mut mistakes = vec![];
        let count = self.inner.len();
        let mut all_ids: HashMap<&str, usize> = HashMap::with_capacity(count);
        let mut all_outputs: HashMap<&Path, usize> = HashMap::with_capacity(count);
        for query in &self.inner {
            mistakes.append(&mut query.validate(query_templates));
            all_ids
                .entry(&query.id)
                .and_modify(|c| *c += 1)
                .or_insert(1);
            if let Some(o) = &query.output {
                all_outputs.entry(o).and_modify(|c| *c += 1).or_insert(1);
            }
        }
        for (key, val) in all_ids.iter() {
            if val > &1 {
                let m = ManifestMistake::Duplicates {
                    key: "queries[].id",
                    value: key,
                };
                mistakes.push(m)
            }
        }
        for (key, val) in all_outputs.iter() {
            if val > &1 {
                let m = ManifestMistake::Duplicates {
                    key: "query_templates[].path",
                    value: key.to_str().unwrap(),
                };
                mistakes.push(m)
            }
        }
        mistakes
    }

    pub fn get(&self, id: &str) -> Option<&Rc<Query>> {
        self.index.get(id)
    }
}
