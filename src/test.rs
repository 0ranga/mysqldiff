#[cfg(test)]
mod tests {
    use diff;
    use std::collections::HashSet;

    #[test]
    fn test() {
        let diff = diff(
            "mysql://root:zaq1xsw2@localhost:3306/template",
            "mysql://root:zaq1xsw2@localhost:3306/imitator",
        );

        let mut expect = HashSet::new();
        expect.insert(
            "CREATE TABLE `ticket` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        );
        expect.insert("DROP TABLE actor;");
        expect.insert("ALTER TABLE `cinema` ADD UNIQUE INDEX 'name_UNIQUE' '(`name`)';");
        expect.insert("ALTER TABLE `user` ADD Role_id int(11) NOT NULL DEFAULT '0';");
        expect.insert("ALTER TABLE `hall` ADD INDEX 'fk_Hall_Cinema1_idx' '(`Cinema_id`)';");
        expect.insert("ALTER TABLE `movie` DROP redundant;");
        expect.insert("ALTER TABLE `movie` MODIFY hallDesignation varchar(500) NOT NULL DEFAULT 'Синий зал';");
        expect.insert("ALTER TABLE `movie` MODIFY genre varchar(200) NOT NULL;");

        assert_eq!(
            diff.iter().map(|s| s.as_ref()).collect::<HashSet<&str>>(),
            expect
        );
    }
}
