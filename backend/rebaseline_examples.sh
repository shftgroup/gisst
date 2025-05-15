#!/usr/bin/env zsh

for table in instance work instancework state save instance_save replay screenshot creator environment file object instanceobject; do
    SQL="select count(*) from $table;"
    sudo -u postgres psql gisstdb -c "$SQL" | sed "s/[[:space:]]//g;3q;d" | xargs printf "sudo -u postgres psql gisstdb -c \"$SQL\" | sed \"s/[[:space:]]//g;3q;d\" | grep -Fx \"%s\"\n"
done
find storage -type f -print0 | du -a --files0-from=- --total | sort -k 2 > du-out.txt
tail -n 1 du-out.txt | cat
wc -l du-out.txt | cut -d' ' -f 1 | cat
