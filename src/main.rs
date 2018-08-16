#[macro_use]
extern crate lazy_static;
//pub mod mysqlex;
extern crate mysql;
extern crate regex;

use my::Pool;
use mysql as my;
use regex::Captures;
use regex::Regex;
use regex::RegexBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;

mod example;

fn main() {
    let pool1 = Pool::new("mysql://root:zaq1xsw2@localhost:3306/differ").unwrap();
    let csql1 = extract_tables(&pool1);
    let diff = diff(&csql1, &csql1);
    diff.iter().for_each(|k| println!("{}", k));
}

fn diff(template: &HashMap<String, Table>, imitator: &HashMap<String, Table>) -> Vec<String> {
    let mut difference = Vec::new();

    let tmps: HashSet<&String> = HashSet::from_iter(template.keys());
    let imis: HashSet<&String> = HashSet::from_iter(imitator.keys());

    let to_create = tmps.difference(&imis);
    to_create.for_each(|k| difference.push(template.get(k.as_str()).unwrap().scheme.clone()));

    let to_drop = imis.difference(&tmps);
    to_drop.for_each(|k| difference.push(format!("DROP TABLE {};", k)));

    let intersection = tmps.intersection(&imis);
    intersection.for_each(
        |k| difference.append(
            diff_table(template.get(k.as_str()).unwrap(),
                       imitator.get(k.as_str()).unwrap())
                .as_mut()));

    difference
}

fn diff_table(template: &Table, imitator: &Table) -> Vec<String> {
    let mut difference = Vec::new();

    difference
}

fn diff_table_columns(template: &Table, imitator: &Table) -> Vec<String> {}

/// extract all table entity from pool
fn extract_tables(pool: &Pool) -> HashMap<String, Table> {
    // show tables
    let show_tbs_ex = pool.prep_exec("SHOW TABLES", ()).unwrap();
    let table_vec: Vec<String> = show_tbs_ex.map(
        |result| my::from_row(result.unwrap())
    ).collect();

    // show create table
    table_vec.iter().map(
        |table| {
            let show_crt_ex = pool.prep_exec(
                format!("SHOW CREATE TABLE {};", table), (),
            ).unwrap();

            show_crt_ex.map(
                |result| my::from_row(result.unwrap())
            ).map(|(col, scheme): (String, String)| (col, scheme.as_str().into())).last().unwrap()
        }).collect()
}

lazy_static! {
    static ref TABLE_BLOCK_PATTERN: Regex = RegexBuilder::new(r"CREATE TABLE .*? ENGINE[^;]*")
                .multi_line(true)
                .dot_matches_new_line(true)
                .build()
                .unwrap();

    static ref TABLE_NAME_PATTERN: Regex = Regex::new(r"`(.*?)`").unwrap();
    static ref PRIMARY_KEY_PATTERN: Regex = Regex::new(r"^\s*PRIMARY KEY\s+\((.*)\)").unwrap();
    static ref UNIQUE_KEY_PATTERN: Regex = Regex::new(r"^\s*UNIQUE KEY\s+`(.*)`\s+\((.*)\)").unwrap();
    static ref ORDINARY_KEY_PATTERN: Regex = Regex::new(r"^\s*KEY\s+`(.*)`\s+\((.*)\)").unwrap();
    static ref COLUMN_PATTERN: Regex = Regex::new(r"^\s*`(.*?)`\s+(.+?)[\n,]?$").unwrap();

    static ref LINT_SPLIT:Regex = Regex::new("\r?\n").unwrap();
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
            None => { return None; }
            Some(t) => t.get(0).unwrap().as_str()
        };

        let table_name = match TABLE_NAME_PATTERN.captures(table_scheme) {
            None => { return None; }
            Some(t) => t.get(0).unwrap().as_str()
        };

        let mut primary_keys: Vec<String> = Vec::new();
        let mut unique_keys: Vec<Key> = Vec::new();
        let mut ordinary_keys: Vec<Key> = Vec::new();
        let mut columns: Vec<Column> = Vec::new();

        LINT_SPLIT.split(table_scheme).filter(
            |l| !(l.starts_with("CREATE") || l.starts_with(')'))
        ).for_each(
            |line| {
                let pkey = PRIMARY_KEY_PATTERN.captures(line);
                if pkey.is_some() {
                    primary_keys.push(pkey.unwrap().get(0).unwrap().as_str().to_string())
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
            }
        );
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
            cpt.get(0).unwrap().as_str().to_string(),
            cpt.get(1).unwrap().as_str().to_string(),
        )
    }
}

impl<'a> From<&'a str> for Table {
    fn from(s: &'a str) -> Table { Table::parse(s).unwrap() }
}