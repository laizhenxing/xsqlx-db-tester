![](https://github.com/laizhenxing/xsqlx-db-tester/workflows/build/badge.svg)

# Source
This project learns from: https://github.com/tyrchen/sqlx-db-tester/tree/master

# xsqlx-db-tester

This a tool test sqlx with postgres. It only support tokio runtime in this moment.

## How to use it

You should first create TestDB data structure in your tests. It will automcatically create a database and a connection pool for you. you cound then get the connection string or connection pool from it to use on your own code. **When TestDB gets drop, it will automcatically drop the database.**

```bash
#[tokio::test]
async fn some_test() {
    let tdb = TestDB::new("postgers://postgrses:postgres@localhost:5432", "./migrations");
    let pool = tdb.get_pool().await;
    // do you test logic

    // the code finish in this test, tdb will drop the database which it create.
}
```

Have fun with this crate!

## License

This project is distributed under the terms of MIT.

See [LICENSE](LICENSE.md) for details.

Copyright 2023 xingxiaoli 
