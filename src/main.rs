#[macro_use]
extern crate lazy_static;
extern crate mysql;
extern crate regex;

use my::Pool;
use mysql as my;
use regex::Captures;
use regex::Regex;
use regex::RegexBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::iter::FromIterator;

mod test;

/// usage cargo run <imitator_uri> <template_uri>
fn main() {
    let args: Vec<String> = env::args().collect();

    let imitator_uri = &args[1];
    let template_uri = &args[2];

    diff(template_uri, imitator_uri);
}

fn diff(template_uri: &str, imitator_uri: &str) -> Vec<String> {
    let template = Pool::new(template_uri).unwrap();
    let imitator = Pool::new(imitator_uri).unwrap();
    let template_tbls = extract_tables(&template);
    let imitator_tbls = extract_tables(&imitator);
    let diff = diff0(&template_tbls, &imitator_tbls);
    diff
}

fn diff0(template: &HashMap<String, Table>, imitator: &HashMap<String, Table>) -> Vec<String> {
    let tmps: HashSet<&String> = HashSet::from_iter(template.keys());
    let imis: HashSet<&String> = HashSet::from_iter(imitator.keys());

    let to_create = tmps.difference(&imis);
    let to_drop = imis.difference(&tmps);
    let intersection = tmps.intersection(&imis);

    let mut difference = Vec::new();
    to_create.for_each(|k| difference.push(template.get(k.as_str()).unwrap().scheme.clone()));
    to_drop.for_each(|k| difference.push(format!("DROP TABLE {};", k)));
    intersection.for_each(|k| {
        difference.append(
            diff_table(
                template.get(k.as_str()).unwrap(),
                imitator.get(k.as_str()).unwrap(),
            ).as_mut(),
        )
    });

    difference
}

fn diff_table(template: &Table, imitator: &Table) -> Vec<String> {
    let mut difference = Vec::new();
    let map_closure = |s: &String| format!("ALTER TABLE {} {};", template.name, s);
    difference.append(
        diff_table_columns(template.columns.as_ref(), imitator.columns.as_ref())
            .iter()
            .map(map_closure)
            .collect::<Vec<String>>()
            .as_mut(),
    );
    difference.append(
        diff_table_keys(template, imitator)
            .iter()
            .map(map_closure)
            .collect::<Vec<String>>()
            .as_mut(),
    );
    difference
}

fn diff_table_columns(template: &Vec<Column>, imitator: &Vec<Column>) -> Vec<String> {
    let tmp_cols: HashSet<&String> = template.iter().map(|c| &(c.0)).collect();
    let imi_cols: HashSet<&String> = imitator.iter().map(|c| &(c.0)).collect();
    let tmp_col_map: HashMap<&String, &String> =
        template.iter().map(|c| (&(c.0), &(c.1))).collect();
    let imi_col_map: HashMap<&String, &String> =
        imitator.iter().map(|c| (&(c.0), &(c.1))).collect();

    let to_add = tmp_cols.difference(&imi_cols);
    let to_drop = imi_cols.difference(&tmp_cols);
    let intersection = tmp_cols.intersection(&imi_cols);

    let mut difference = Vec::new();
    to_add.for_each(|c| difference.push(format!("ADD {} {}", c, tmp_col_map.get(c).unwrap())));
    to_drop.for_each(|c| difference.push(format!("DROP {}", c)));
    intersection.for_each(|c| {
        if tmp_col_map.get(c) != imi_col_map.get(c) {
            difference.push(format!("MODIFY {} {}", c, tmp_col_map.get(c).unwrap()));
        }
    });
    difference
}

fn diff_table_keys(template: &Table, imitator: &Table) -> Vec<String> {
    let mut difference = Vec::new();
    difference.append(
        diff_table_ordinary_keys(
            template.ordinary_keys.as_ref(),
            imitator.ordinary_keys.as_ref(),
            |t: &Key| format!("ADD INDEX '{}' '({})'", t.0, t.1),
            |t: &Key| format!("DROP INDEX '{}'", t.0),
        ).as_mut(),
    );

    difference.append(
        diff_table_ordinary_keys(
            template.unique_keys.as_ref(),
            imitator.unique_keys.as_ref(),
            |t: &Key| format!("ADD UNIQUE INDEX '{}' '({})'", t.0, t.1),
            |t: &Key| format!("DROP INDEX '{}'", t.0),
        ).as_mut(),
    );
    difference
}

// if not set G like F, compiler will complain "no two closures, even if identical, have the same type"
// or use Box
fn diff_table_ordinary_keys<F, G>(
    template: &Vec<Key>,
    imitator: &Vec<Key>,
    add_format: F,
    drop_format: G,
) -> Vec<String>
where
    F: Fn(&StringTuple) -> String,
    G: Fn(&StringTuple) -> String,
{
    let mut difference = Vec::new();
    let tmp_keys: HashSet<&String> = template.iter().map(|c| &(c.1)).collect();
    let imi_keys: HashSet<&String> = imitator.iter().map(|c| &(c.1)).collect();
    template.iter().for_each(|t| {
        if !imi_keys.contains(&(t.1)) {
            difference.push(add_format(t));
        }
    });

    imitator.iter().for_each(|t| {
        if !tmp_keys.contains(&(t.1)) {
            difference.push(drop_format(t));
        }
    });
    difference
}

/// extract all table entity from pool
fn extract_tables(pool: &Pool) -> HashMap<String, Table> {
    // show tables
    let show_tbs_ex = pool.prep_exec("SHOW TABLES", ()).unwrap();
    let table_vec: Vec<String> = show_tbs_ex
        .map(|result| my::from_row(result.unwrap()))
        .collect();

    // show create table
    table_vec
        .iter()
        .map(|table| {
            let show_crt_ex = pool
                .prep_exec(format!("SHOW CREATE TABLE {};", table), ())
                .unwrap();

            show_crt_ex
                .map(|result| my::from_row(result.unwrap()))
                .map(|(col, scheme): (String, String)| (col, scheme.as_str().into()))
                .last()
                .unwrap()
        })
        .collect()
}

lazy_static! {
    static ref TABLE_BLOCK_PATTERN: Regex = RegexBuilder::new(r"CREATE TABLE .*? ENGINE[^;]*")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();
    static ref TABLE_NAME_PATTERN: Regex = Regex::new(r"`(.*?)`").unwrap();
    static ref PRIMARY_KEY_PATTERN: Regex = Regex::new(r"^\s*PRIMARY KEY\s+\((.*)\)").unwrap();
    static ref UNIQUE_KEY_PATTERN: Regex =
        Regex::new(r"^\s*UNIQUE KEY\s+`(.*)`\s+\((.*)\)").unwrap();
    static ref ORDINARY_KEY_PATTERN: Regex = Regex::new(r"^\s*KEY\s+`(.*)`\s+\((.*)\)").unwrap();
    static ref COLUMN_PATTERN: Regex = Regex::new(r"^\s*`(.*?)`\s+(.+?)[\n,]?$").unwrap();
    static ref LINT_SPLIT: Regex = Regex::new("\r?\n").unwrap();
}

#[derive(Debug)]
struct StringTuple(String, String);

type Column = StringTuple;
type Key = StringTuple;

#[derive(Debug)]
struct Table {
    name: String,
    primary_keys: Vec<String>,
    unique_keys: Vec<Key>,
    ordinary_keys: Vec<Key>,
    columns: Vec<Column>,
    scheme: String,
}

impl Table {
    fn parse(scheme: &str) -> Option<Table> {
        let matches = TABLE_BLOCK_PATTERN.captures(scheme);
        let table_scheme = match matches {
            None => return None,
            Some(t) => t.get(0).unwrap().as_str(),
        };

        let table_name = match TABLE_NAME_PATTERN.captures(table_scheme) {
            None => return None,
            Some(t) => t.get(0).unwrap().as_str(),
        };

        let mut primary_keys: Vec<String> = Vec::new();
        let mut unique_keys: Vec<Key> = Vec::new();
        let mut ordinary_keys: Vec<Key> = Vec::new();
        let mut columns: Vec<Column> = Vec::new();

        LINT_SPLIT
            .split(table_scheme)
            .filter(|l| !(l.starts_with("CREATE") || l.starts_with(')')))
            .for_each(|line| {
                let pkey = PRIMARY_KEY_PATTERN.captures(line);
                if pkey.is_some() {
                    let x = pkey.unwrap().get(1).unwrap().as_str().to_string();
                    primary_keys.push(x)
                }

                let ukey = UNIQUE_KEY_PATTERN.captures(line);
                if ukey.is_some() {
                    unique_keys.push(Table::gen_tuple(ukey.unwrap()))
                }

                let okey = ORDINARY_KEY_PATTERN.captures(line);
                if okey.is_some() {
                    ordinary_keys.push(Table::gen_tuple(okey.unwrap()))
                }

                let col = COLUMN_PATTERN.captures(line);
                if col.is_some() {
                    columns.push(Table::gen_tuple(col.unwrap()))
                }
            });
        Some(Table {
            name: table_name.to_string(),
            primary_keys,
            unique_keys,
            ordinary_keys,
            columns,
            scheme: table_scheme.to_string(),
        })
    }

    fn gen_tuple(cpt: Captures) -> StringTuple {
        StringTuple(
            cpt.get(1).unwrap().as_str().trim().to_string(),
            cpt.get(2).unwrap().as_str().trim().to_string(),
        )
    }
}

impl<'a> From<&'a str> for Table {
    fn from(s: &'a str) -> Table {
        Table::parse(s).unwrap()
    }
}
