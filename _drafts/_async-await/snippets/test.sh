#!/bin/sh

code=0

for file in *.rs
do
    printf "Testing %-30s…" "$file"

    output=$(rustfmt --check --color=always --edition=2018 "$file")
    if [ "$?" != "0" ]
    then
        echo "fail, style error:\n$output"
        code=1
        continue
    fi

    echo "ok"
done

exit $code
