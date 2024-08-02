
# Ceres

## How to setup your project

1. Change project name
    - `Cargo.toml` package name
    - `codefresh.yaml` variables
2. [Setup Codefresh pipeline](https://instapartners.atlassian.net/wiki/spaces/PP/pages/1350402769/New+Codefresh+Pipeline)
3. Setup Kubernetes secrets in `default` namespace for QA, and `prod` namespace for production.
4. Create Postgres user for the database, and then modify the `setup_db_users` migration accordingly. (This migration does not create users, but only assigns permissions and roles to them)

## Integration tests

In the folder `tests` is available an example of an integration tests using the database. In order to run these tests, it is required to have a running instance of Postgres available.

`codefresh.yaml` includes a step in which it starts these tests in a Docker compose environment, after performing the migrations.


### Run tests in local environment

First launch Docker Compose in order to start Postgres and migrations:
```
docker-compose up
```

Then launch tests:
```
./run_tests.sh
```
