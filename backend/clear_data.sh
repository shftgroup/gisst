# Check that sqlx-cli is installed
if ! command -v sqlx &> /dev/null
then
  echo "sqlx-cli could not be found"
  exit 1
fi

# Check that postgres cli (psql) is installed
if ! command -v psql &> /dev/null
then
  echo "psql could not be found. Is PostgreSQL installed?"
  exit 1
fi

# Remove local file storage
rm -r storage
# Remove test_data load, changed to blowing away test database
# See https://github.com/shftgroup/gisst/issues/106 if you want to work on fixing this
#sqlx migrate revert # remove null
#sqlx migrate revert # remove users
#sqlx migrate revert # remove insert data
#sqlx migrate revert # remove base tables
#sqlx migrate revert # remove creator changes

# Remove and recreate test database
echo "Dropping database gisstdb."
psql -U postgres -c "drop database gisstdb"
echo "Creating database gisstdb."
psql -U postgres -c "create database gisstdb"

# Recreate tables and data load
sqlx migrate run
