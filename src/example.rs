#[cfg(test)]
mod tests {
    use mysql as my;
    use std::env;

    // usage:
    //    mysql_uri="mysql://root:zaq1xsw2@localhost:3306/differ" cargo test --all -- --nocapture
    #[test]
    fn test() {
//        let mysql_uri = env::var("mysql_uri").unwrap();
//        let pool = my::Pool::new(mysql_uri).unwrap();
//        pool.prep_exec(r"CREATE TEMPORARY TABLE payment (
//                         customer_id int not null,
//                         amount int not null,
//                         account_name text
//                     )", ()).unwrap();
//
//        println!("{}", mysql_uri);
    }
}