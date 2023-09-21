# Remove local file storage
rm -r storage
# Remove test_data load
sqlx migrate revert # remove users
sqlx migrate revert # remove insert data
sqlx migrate revert # remove base tables

# Recreate tables and data load
sqlx migrate run
