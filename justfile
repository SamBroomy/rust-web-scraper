# use `jsut watch` to run tests on file save
run:
    cargo run

db:
    surreal start file:mydb.db --log trace -A --auth --user root --pass root --bind 0.0.0.0:8080

quick_dev:
    cargo watch -q -c -w examples/ -x "run --example quick_dev"

watch:
    cargo watch -q -c -w . -x test

install:
    curl -sSf https://install.surrealdb.com | sh