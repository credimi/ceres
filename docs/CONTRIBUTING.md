## Integration tests

In the folder `tests` is available an example of an integration tests using the database.
In order to run these tests, it is required to have a running instance of Postgres available.

The pipeline file (`codefresh.yaml`) includes a step in which it starts these tests in a Docker compose environment,
after performing the migrations.

### Run tests in local environment

First launch Docker Compose (`docker-compose up`) in order to start Postgres and migrations, then launch tests with `cargo test`.