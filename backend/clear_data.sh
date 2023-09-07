# Remove local file storage
rm -r storage
# Remove test_data load
sqlx migrate revert
# Remove tables and data
sqlx migrate revert
# Recreate tables and data load
sqlx migrate run