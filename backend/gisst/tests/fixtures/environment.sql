-- Depends on: core.sql
INSERT INTO environment (
    environment_id,
    environment_name,
    environment_framework,
    environment_platform,
    environment_core_name,
    environment_core_version
) VALUES (
'00000000-0000-0000-0000-000000000001',
'Test Environment',
'retroarch',
'Test Platform',
'original-core',
'1.0'
);